use crate::events::TVLFeePaid;
use crate::state::{FeeDistribution, Infinix};
use crate::utils::structs::InfinixStatus;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_2022;
use anchor_spl::token_interface::{self, Mint, TokenInterface};
use shared::check_condition;
use shared::constants::{FEE_DISTRIBUTION_SEEDS, INFINIX_SEEDS, MAX_FEE_RECIPIENTS_PORTION};
use shared::errors::ErrorCode;
use shared::utils::account_util::next_account;
use shared::utils::{Decimal, Rounding};

#[derive(Accounts)]
pub struct CrankFeeDistribution<'info> {
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Cranker account
    #[account(mut)]
    pub cranker: UncheckedAccount<'info>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut)]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub fee_distribution: AccountLoader<'info, FeeDistribution>,
    /*
    Remaining accounts will be the token accounts of the fee recipients, needs to follow the
    order of the indices passed as parameters.
     */
}

impl CrankFeeDistribution<'_> {
    pub fn validate(&self, infinix: &Infinix, fee_distribution: &FeeDistribution) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            None,
            None,
            Some(vec![InfinixStatus::Initialized, InfinixStatus::Killed]),
        )?;

        // Validate fee distribution
        check_condition!(
            self.fee_distribution.key()
                == Pubkey::find_program_address(
                    &[
                        FEE_DISTRIBUTION_SEEDS,
                        self.infinix.key().as_ref(),
                        fee_distribution.index.to_le_bytes().as_slice()
                    ],
                    &crate::id()
                )
                .0,
            InvalidFeeDistribution
        );

        check_condition!(
            self.infinix_token_mint.key() == infinix.infinix_token_mint,
            InvalidInfinixTokenMint
        );

        check_condition!(
            self.cranker.key() == fee_distribution.cranker,
            InvalidCranker
        );

        Ok(())
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, CrankFeeDistribution<'info>>,
    indices: Vec<u64>,
) -> Result<()> {
    let infinix_bump: u8;
    let scaled_total_amount_to_distribute: u128;

    let token_mint_key = ctx.accounts.infinix_token_mint.key();

    {
        let infinix = &ctx.accounts.infinix.load()?;

        let fee_distribution = &ctx.accounts.fee_distribution.load()?;

        infinix_bump = infinix.bump;
        scaled_total_amount_to_distribute = fee_distribution.amount_to_distribute;

        ctx.accounts.validate(infinix, fee_distribution)?;
    }

    let signer_seeds = &[INFINIX_SEEDS, token_mint_key.as_ref(), &[infinix_bump]];

    let mut amount_to_remove_from_infinix_pending_fees: u128 = 0;

    let remaining_accounts = &ctx.remaining_accounts;
    let mut remaining_accounts_iter = remaining_accounts.iter();
    {
        let fee_distribution = &mut ctx.accounts.fee_distribution.load_mut()?;
        for index in indices {
            let fee_recipient = next_account(
                &mut remaining_accounts_iter,
                false,
                true,
                ctx.accounts.token_program.key,
            )?;

            let related_fee_distribution =
                &mut fee_distribution.fee_recipients_state[index as usize];

            // Already distributed (set as default pubkey when distributed)
            if related_fee_distribution.recipient.key() == Pubkey::default() {
                continue;
            }

            // Validate proper token account for the recipient
            check_condition!(
                fee_recipient.key()
                    == get_associated_token_address_with_program_id(
                        &related_fee_distribution.recipient.key(),
                        &ctx.accounts.infinix_token_mint.key(),
                        &ctx.accounts.token_program.key(),
                    ),
                InvalidFeeRecipient
            );

            // Set as distributed
            related_fee_distribution.recipient = Pubkey::default();

            let raw_amount_to_distribute = Decimal::from_scaled(scaled_total_amount_to_distribute)
                .mul(&Decimal::from_scaled(related_fee_distribution.portion))?
                .div(&Decimal::from_scaled(MAX_FEE_RECIPIENTS_PORTION))?
                .to_token_amount(Rounding::Floor)?
                .0;

            let cpi_accounts = token_2022::MintTo {
                mint: ctx.accounts.infinix_token_mint.to_account_info(),
                to: fee_recipient.to_account_info(),
                authority: ctx.accounts.infinix.to_account_info(),
            };

            token_interface::mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    cpi_accounts,
                    &[signer_seeds],
                ),
                raw_amount_to_distribute,
            )?;

            amount_to_remove_from_infinix_pending_fees = amount_to_remove_from_infinix_pending_fees
                .checked_add(raw_amount_to_distribute as u128)
                .ok_or(ErrorCode::MathOverflow)?;

            emit!(TVLFeePaid {
                recipient: related_fee_distribution.recipient.key(),
                amount: raw_amount_to_distribute,
            });
        }
    }

    // Check if we can close the fee distribution account to reimburse the cranker for the rent
    let mut can_close = false;
    {
        let fee_distribution = &ctx.accounts.fee_distribution.load()?;
        if fee_distribution.is_fully_distributed() {
            can_close = true;
        }
    }

    if can_close {
        ctx.accounts
            .fee_distribution
            .close(ctx.accounts.cranker.to_account_info())?;
    }
    let scaled_amount_to_remove_from_infinix_pending_fees =
        Decimal::from_token_amount(amount_to_remove_from_infinix_pending_fees)?
            .to_scaled(Rounding::Floor)?;

    let infinix = &mut ctx.accounts.infinix.load_mut()?;
    infinix.fee_recipients_pending_fee_shares_to_be_minted = infinix
        .fee_recipients_pending_fee_shares_to_be_minted
        .checked_sub(scaled_amount_to_remove_from_infinix_pending_fees)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(())
}
