use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id,
    token_interface::{self, Mint, TokenInterface, TransferChecked},
};
use shared::errors::ErrorCode;
use shared::{
    check_condition,
    constants::{INFINIX_BASKET_SEEDS, USER_PENDING_BASKET_SEEDS},
};

use crate::state::{Infinix, InfinixBasket, UserPendingBasket};
use crate::utils::structs::{InfinixStatus, TokenAmount};
use shared::utils::account_util::next_account;

const EXPECTED_REMAINING_ACCOUNTS_LENGTH: usize = 3;

#[derive(Accounts)]
pub struct AddToPendingBasket<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account()]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut,
        seeds = [INFINIX_BASKET_SEEDS, infinix.key().as_ref()],
        bump
    )]
    pub infinix_basket: AccountLoader<'info, InfinixBasket>,

    #[account(init_if_needed,
        payer = user,
        space = UserPendingBasket::SIZE,
        seeds = [USER_PENDING_BASKET_SEEDS, infinix.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_pending_basket: AccountLoader<'info, UserPendingBasket>,
}

impl AddToPendingBasket<'_> {
    pub fn validate(&self) -> Result<()> {
        let infinix = self.infinix.load()?;
        infinix.validate_infinix(
            &self.infinix.key(),
            None,
            None,
            Some(vec![InfinixStatus::Initialized]),
        )?;

        Ok(())
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, AddToPendingBasket<'info>>,
    raw_amounts: Vec<u64>,
) -> Result<()> {
    ctx.accounts.validate()?;

    let remaining_accounts = &ctx.remaining_accounts;
    let mut remaining_accounts_iter = remaining_accounts.iter();

    let infinix_key = ctx.accounts.infinix.key();
    let token_program_id = ctx.accounts.token_program.key();
    let user = ctx.accounts.user.to_account_info();

    check_condition!(
        remaining_accounts.len() % EXPECTED_REMAINING_ACCOUNTS_LENGTH == 0,
        InvalidNumberOfRemainingAccounts
    );

    check_condition!(
        remaining_accounts.len() / EXPECTED_REMAINING_ACCOUNTS_LENGTH == raw_amounts.len(),
        InvalidNumberOfRemainingAccounts
    );

    let mut added_mints: Vec<TokenAmount> = vec![];

    for raw_amount in raw_amounts {
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

        // Validate the recipient token account is the ATA of the infinix
        check_condition!(
            recipient_token_account.key()
                == get_associated_token_address_with_program_id(
                    &infinix_key,
                    token_mint.key,
                    &token_program_id,
                ),
            InvalidRecipientTokenAccount
        );

        // Get decimals from token mint
        let data = token_mint.try_borrow_data()?;
        let mint = Mint::try_deserialize(&mut &data[..])?;

        let cpi_accounts = TransferChecked {
            from: sender_token_account.to_account_info(),
            to: recipient_token_account.to_account_info(),
            authority: user.clone(),
            mint: token_mint.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        token_interface::transfer_checked(
            CpiContext::new(cpi_program, cpi_accounts),
            raw_amount,
            mint.decimals,
        )?;

        added_mints.push(TokenAmount {
            mint: token_mint.key(),
            amount_for_minting: raw_amount,
            amount_for_redeeming: 0,
        });
    }

    UserPendingBasket::process_init_if_needed(
        &mut ctx.accounts.user_pending_basket,
        ctx.bumps.user_pending_basket,
        &ctx.accounts.user.key(),
        &ctx.accounts.infinix.key(),
        &added_mints,
        true,
    )?;

    Ok(())
}
