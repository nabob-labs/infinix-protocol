use crate::state::{Infinix, InfinixBasket, UserPendingBasket};
use crate::utils::structs::TokenAmount;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id,
    token_interface::{self, Mint, TokenInterface, TransferChecked},
};
use shared::errors::ErrorCode;
use shared::utils::account_util::next_account;
use shared::{
    check_condition,
    constants::{PendingBasketType, INFINIX_BASKET_SEEDS, INFINIX_SEEDS, USER_PENDING_BASKET_SEEDS},
};

const EXPECTED_REMAINING_ACCOUNTS_LENGTH: usize = 3;

#[derive(Accounts)]
pub struct RemoveFromPendingBasket<'info> {
    pub system_program: Program<'info, System>,
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

    #[account(mut,
        seeds = [USER_PENDING_BASKET_SEEDS, infinix.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_pending_basket: AccountLoader<'info, UserPendingBasket>,
}

impl RemoveFromPendingBasket<'_> {
    pub fn validate(&self, infinix: &Infinix) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            None,
            None,
            // User should always be able to take back their pending tokens
            None,
        )?;

        Ok(())
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, RemoveFromPendingBasket<'info>>,
    raw_amounts: Vec<u64>,
) -> Result<()> {
    let infinix_info = ctx.accounts.infinix.to_account_info();
    let infinix = ctx.accounts.infinix.load()?;

    ctx.accounts.validate(&infinix)?;

    let remaining_accounts = &ctx.remaining_accounts;
    let mut remaining_accounts_iter = remaining_accounts.iter();

    let user_key = ctx.accounts.user.key();
    let token_program_id = ctx.accounts.token_program.key();

    check_condition!(
        remaining_accounts.len() % EXPECTED_REMAINING_ACCOUNTS_LENGTH == 0,
        InvalidNumberOfRemainingAccounts
    );

    check_condition!(
        remaining_accounts.len() / EXPECTED_REMAINING_ACCOUNTS_LENGTH == raw_amounts.len(),
        InvalidNumberOfRemainingAccounts
    );

    let mut removed_mints: Vec<TokenAmount> = vec![];

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
                    &user_key,
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
            authority: infinix_info.clone(),
            mint: token_mint.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();

        let infinix_mint_key = infinix.infinix_token_mint;
        let signer_seeds = &[INFINIX_SEEDS, infinix_mint_key.as_ref(), &[infinix.bump]];

        token_interface::transfer_checked(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, &[signer_seeds]),
            raw_amount,
            mint.decimals,
        )?;

        removed_mints.push(TokenAmount {
            mint: token_mint.key(),
            amount_for_minting: raw_amount,
            amount_for_redeeming: 0,
        });
    }

    ctx.accounts
        .user_pending_basket
        .load_mut()?
        .remove_token_amounts_from_infinix(&removed_mints, true, PendingBasketType::MintProcess)?;

    Ok(())
}
