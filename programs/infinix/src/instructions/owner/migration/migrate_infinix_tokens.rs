use crate::utils::NewInfinixProgram;
use crate::ID as INFINIX_PROGRAM_ID;
use crate::{
    state::{Infinix, InfinixBasket},
    utils::InfinixStatus,
};
use anchor_lang::{prelude::*, Discriminator};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_interface;
use anchor_spl::token_interface::{Mint, TokenInterface, TransferChecked};
use infinix_admin::{state::ProgramRegistrar, ID as INFINIX_ADMIN_PROGRAM_ID};
use shared::errors::ErrorCode;
use shared::utils::account_util::next_account;
use shared::{
    check_condition,
    constants::{INFINIX_BASKET_SEEDS, INFINIX_SEEDS, PROGRAM_REGISTRAR_SEEDS},
};

const REMAINING_ACCOUNTS_DIVIDER: usize = 3;

#[derive(Accounts)]
pub struct MigrateInfinixTokens<'info> {
    pub token_program: Interface<'info, TokenInterface>,

    // Is permissionless, so infinix isn't blocked by infinix owner.
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [PROGRAM_REGISTRAR_SEEDS],
        bump = program_registrar.bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub program_registrar: Box<Account<'info, ProgramRegistrar>>,

    /// CHECK: Infinix program used for new infinix
    #[account(executable)]
    pub new_infinix_program: UncheckedAccount<'info>,

    #[account()]
    pub old_infinix: AccountLoader<'info, Infinix>,

    #[account(
        mut,
        seeds = [INFINIX_BASKET_SEEDS, old_infinix.key().as_ref()],
        bump
    )]
    pub old_infinix_basket: AccountLoader<'info, InfinixBasket>,

    /// CHECK: Seeds are checked and the account data is checked in cpi to new infinix program
    #[account(
        mut,
        seeds = [INFINIX_BASKET_SEEDS, new_infinix.key().as_ref()],
        bump,
        seeds::program = new_infinix_program.key(),
    )]
    pub new_infinix_basket: UncheckedAccount<'info>,

    /// CHECK: The new infinix
    #[account(mut)]
    pub new_infinix: UncheckedAccount<'info>,

    // Validate mint is now owned by the new infinix
    #[account(
        mint::authority = new_infinix,
        mint::freeze_authority = new_infinix,
    )]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl MigrateInfinixTokens<'_> {
    pub fn validate(&self, old_infinix: &Infinix) -> Result<()> {
        // Validate old infinix
        old_infinix.validate_infinix(
            &self.old_infinix.key(),
            None,
            None,
            Some(vec![InfinixStatus::Migrating]),
        )?;

        check_condition!(
            old_infinix.infinix_token_mint == self.infinix_token_mint.key(),
            InvalidInfinixTokenMint
        );

        /*
        New Folio Validation
         */
        // Make sure the new folio program is in the registrar
        check_condition!(
            self.program_registrar
                .is_in_registrar(self.new_infinix_program.key()),
            ProgramNotInRegistrar
        );

        // Make sure the new infinix is owned by the new infinix program
        check_condition!(
            *self.new_infinix.owner == self.new_infinix_program.key(),
            NewInfinixNotOwnedByNewInfinixProgram
        );

        check_condition!(
            self.new_infinix_program.key() != INFINIX_PROGRAM_ID,
            CantMigrateToSameProgram
        );

        // Make sure the discriminator of the new infinix is correct
        let data = self.new_infinix.try_borrow_data()?;
        // check_condition!(
        // data.len() >= 8 && data[0..8] == Infinix::discriminator(),
        //    InvalidNewInfinix
        // );

        Ok(())
    }
}

pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info, MigrateInfinixTokens<'info>>) -> Result<()> {
    let old_infinix_key = ctx.accounts.old_infinix.key();
    let new_infinix_key = ctx.accounts.new_infinix.key();
    let token_program_id = ctx.accounts.token_program.key();

    let old_infinix_token_mint: Pubkey;
    let old_infinix_bump: u8;

    {
        let old_infinix = &ctx.accounts.old_infinix.load()?;

        old_infinix_token_mint = old_infinix.infinix_token_mint;
        old_infinix_bump = old_infinix.bump;

        ctx.accounts.validate(old_infinix)?;
    }

    let infinix_signer_seeds = &[
        INFINIX_SEEDS,
        old_infinix_token_mint.as_ref(),
        &[old_infinix_bump],
    ];
    let infinix_signer = &[&infinix_signer_seeds[..]];

    /*
    Transfer the infinix tokens (from the infinix basket), won't transfer the pending amounts, as those users
    will be able to take them back, on the old infinix program, rather than the new one for simplicity and security.
    */
    check_condition!(
        ctx.remaining_accounts.len() % REMAINING_ACCOUNTS_DIVIDER == 0,
        InvalidNumberOfRemainingAccounts
    );

    let mut remaining_accounts_iter = ctx.remaining_accounts.iter();

    for _ in 0..ctx.remaining_accounts.len() / REMAINING_ACCOUNTS_DIVIDER {
        let token_mint = next_account(
            &mut remaining_accounts_iter,
            false,
            false,
            &token_program_id,
        )?;
        let sender_token_account =
            next_account(&mut remaining_accounts_iter, false, true, &token_program_id)?;
        let recipient_token_account =
            next_account(&mut remaining_accounts_iter, false, true, &token_program_id)?;

        // Validate the sender token account is the ATA of the old infinix
        check_condition!(
            sender_token_account.key()
                == get_associated_token_address_with_program_id(
                    &old_infinix_key,
                    token_mint.key,
                    &token_program_id,
                ),
            InvalidSenderTokenAccount
        );

        // Validate the recipient token account is the ATA of the new infinix
        check_condition!(
            recipient_token_account.key()
                == get_associated_token_address_with_program_id(
                    &new_infinix_key,
                    token_mint.key,
                    &token_program_id,
                ),
            InvalidRecipientTokenAccount
        );

        let raw_migrate_balance: u64;

        {
            let old_infinix_basket = &ctx.accounts.old_infinix_basket.load()?;

            raw_migrate_balance =
                old_infinix_basket.get_token_amount_in_infinix_basket(token_mint.key)?;
        }

        let mint_decimals = {
            let data = token_mint.try_borrow_data()?;
            Mint::try_deserialize(&mut &data[..])?.decimals
        };

        let cpi_accounts = TransferChecked {
            from: sender_token_account.to_account_info(),
            to: recipient_token_account.to_account_info(),
            authority: ctx.accounts.old_infinix.to_account_info(),
            mint: token_mint.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        token_interface::transfer_checked(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, infinix_signer),
            raw_migrate_balance,
            mint_decimals,
        )?;

        NewInfinixProgram::update_infinix_basket_in_new_infinix_program(
            &ctx.accounts.old_infinix.to_account_info(),
            &ctx.accounts.new_infinix.to_account_info(),
            &ctx.accounts.old_infinix_basket.to_account_info(),
            &ctx.accounts.new_infinix_basket.to_account_info(),
            &token_mint.to_account_info(),
            &recipient_token_account.to_account_info(),
            &ctx.accounts.program_registrar.to_account_info(),
            &ctx.accounts.new_infinix_program.to_account_info(),
            &[&infinix_signer_seeds[..]],
        )?;

        {
            let old_infinix_basket = &mut ctx.accounts.old_infinix_basket.load_mut()?;
            // Remove the token from the old infinix basket
            old_infinix_basket.remove_token_mint_from_basket(token_mint.key())?;
        }
    }

    Ok(())
}
