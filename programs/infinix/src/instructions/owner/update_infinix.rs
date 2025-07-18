use crate::events::{AuctionLengthSet, MintFeeSet};

use crate::instructions::distribute_fees;
use crate::state::{Actor, FeeDistribution, FeeRecipients, Infinix};
use crate::utils::structs::{FeeRecipient, Role};
use crate::utils::{FixedSizeString, InfinixStatus, MAX_PADDED_STRING_LENGTH};
use crate::ID;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_spl::token_interface::Mint;
use shared::constants::{
    FEE_DISTRIBUTION_SEEDS, FEE_RECIPIENTS_SEEDS, MAX_AUCTION_LENGTH, MAX_MINT_FEE, MAX_TVL_FEE,
    MIN_AUCTION_LENGTH,
};
use shared::errors::ErrorCode;
use shared::utils::init_pda_account_rent;
use shared::{check_condition, constants::ACTOR_SEEDS};

enum IndexPerAccount {
    TokenProgram,
    DAOFeeConfig,
    InfinixFeeConfig,
    InfinixTokenMint,
    FeeDistribution,
    DAOFeeRecipient,
}

#[derive(Accounts)]
pub struct UpdateInfinix<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub infinix_owner: Signer<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, infinix_owner.key().as_ref(), infinix.key().as_ref()],
        bump = actor.bump,
    )]
    pub actor: Account<'info, Actor>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(
        init_if_needed,
        space = FeeRecipients::SIZE,
        payer = infinix_owner,
        seeds = [FEE_RECIPIENTS_SEEDS, infinix.key().as_ref()],
        bump,
    )]
    pub fee_recipients: AccountLoader<'info, FeeRecipients>,
}

impl UpdateInfinix<'_> {
    pub fn validate(&self) -> Result<()> {
        {
            let infinix = self.infinix.load()?;
            infinix.validate_infinix(
                &self.infinix.key(),
                Some(&self.actor),
                Some(vec![Role::Owner]),
                None, // Can update no matter the status
            )?;
        }

        Ok(())
    }
}

impl<'info> UpdateInfinix<'info> {
    pub fn distribute_fees(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
        index_for_fee_distribution: Option<u64>,
    ) -> Result<()> {
        {
            let infinix_status = {
                let infinix = self.infinix.load()?;
                infinix.status.into()
            };

            // Don't distribute fees if the isn't INITIALIZED or KILLED
            if ![InfinixStatus::Killed, InfinixStatus::Initialized].contains(&infinix_status) {
                return Ok(());
            }

            if index_for_fee_distribution.is_none() {
                return Err(error!(ErrorCode::MissingFeeDistributionIndex));
            }

            let dao_fee_config =
                Account::try_from(&remaining_accounts[IndexPerAccount::DAOFeeConfig as usize])?;

            let infinix_token_mint: Box<InterfaceAccount<Mint>> =
                Box::new(InterfaceAccount::try_from(
                    &remaining_accounts[IndexPerAccount::InfinixTokenMint as usize],
                )?);

            // Create the fee distribution account (since the distribute fees init it, but we're skipping the anchor's context by
            // calling the function directly)
            let infinix_key = self.infinix.key();
            let index_for_fee_distribution_parsed =
                index_for_fee_distribution.unwrap().to_le_bytes();

            let seeds_for_fee_distribution = &[
                FEE_DISTRIBUTION_SEEDS,
                infinix_key.as_ref(),
                index_for_fee_distribution_parsed.as_slice(),
            ];

            let (fee_distribution_account, fee_distribution_bump) =
                Pubkey::find_program_address(seeds_for_fee_distribution, &ID);

            let seeds_with_bump = [
                FEE_DISTRIBUTION_SEEDS,
                infinix_key.as_ref(),
                index_for_fee_distribution_parsed.as_slice(),
                &[fee_distribution_bump],
            ];

            check_condition!(
                fee_distribution_account
                    == remaining_accounts[IndexPerAccount::FeeDistribution as usize].key(),
                InvalidFeeDistribution
            );

            init_pda_account_rent(
                &remaining_accounts[IndexPerAccount::FeeDistribution as usize],
                FeeDistribution::SIZE,
                &self.infinix_owner,
                &ID,
                &self.system_program,
                &[&seeds_with_bump[..]],
            )?;

            let fee_distribution: AccountLoader<FeeDistribution> =
                AccountLoader::try_from_unchecked(
                    &system_program::ID,
                    &remaining_accounts[IndexPerAccount::FeeDistribution as usize],
                )?;

            distribute_fees(
                &remaining_accounts[IndexPerAccount::TokenProgram as usize],
                &self.infinix_owner,
                &dao_fee_config,
                &remaining_accounts[IndexPerAccount::InfinixFeeConfig as usize],
                &self.infinix,
                &infinix_token_mint,
                &self.fee_recipients,
                &fee_distribution,
                &remaining_accounts[IndexPerAccount::DAOFeeRecipient as usize],
                index_for_fee_distribution.unwrap(),
            )?;
        }

        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, UpdateInfinix<'info>>,
    scaled_tvl_fee: Option<u128>,
    // Only needed if we need to distribute the fees
    index_for_fee_distribution: Option<u64>,
    scaled_mint_fee: Option<u128>,
    auction_length: Option<u64>,
    fee_recipients_to_add: Vec<FeeRecipient>,
    fee_recipients_to_remove: Vec<Pubkey>,
    mandate: Option<String>,
) -> Result<()> {
    ctx.accounts.validate()?;

    let mut should_distribute_fees: bool;
    // Only distribute fees if the fee recipients account is already initialized
    {
        should_distribute_fees = !FeeRecipients::process_init_if_needed(
            &mut ctx.accounts.fee_recipients,
            ctx.bumps.fee_recipients,
            &ctx.accounts.infinix.key(),
        )?;
    }

    if let Some(scaled_tvl_fee) = scaled_tvl_fee {
        check_condition!(scaled_tvl_fee <= MAX_TVL_FEE, TVLFeeTooHigh);

        if should_distribute_fees {
            ctx.accounts
                .distribute_fees(ctx.remaining_accounts, index_for_fee_distribution)?;

            // Don't want to distribute twice
            should_distribute_fees = false;
        }

        {
            let mut infinix = ctx.accounts.infinix.load_mut()?;
            infinix.set_tvl_fee(scaled_tvl_fee)?;
        }
    }

    if let Some(scaled_mint_fee) = scaled_mint_fee {
        check_condition!(scaled_mint_fee <= MAX_MINT_FEE, InvalidMintFee);

        if should_distribute_fees {
            ctx.accounts
                .distribute_fees(ctx.remaining_accounts, index_for_fee_distribution)?;

            // Don't want to distribute twice
            should_distribute_fees = false;
        }

        {
            let mut infinix = ctx.accounts.infinix.load_mut()?;
            infinix.mint_fee = scaled_mint_fee;
        }

        emit!(MintFeeSet {
            new_fee: scaled_mint_fee
        });
    }

    if !fee_recipients_to_add.is_empty() || !fee_recipients_to_remove.is_empty() {
        if should_distribute_fees {
            ctx.accounts
                .distribute_fees(ctx.remaining_accounts, index_for_fee_distribution)?;
        }

        {
            let fee_recipients_res = &mut ctx.accounts.fee_recipients.load_mut();
            let fee_recipients = match fee_recipients_res {
                Ok(fee_recipients) => fee_recipients,
                Err(_) => &mut ctx.accounts.fee_recipients.load_init()?,
            };

            fee_recipients
                .update_fee_recipients(fee_recipients_to_add, fee_recipients_to_remove)?;
        }
    }

    if let Some(auction_length) = auction_length {
        check_condition!(
            (MIN_AUCTION_LENGTH..=MAX_AUCTION_LENGTH).contains(&auction_length),
            InvalidAuctionLength
        );

        {
            let mut infinix = ctx.accounts.infinix.load_mut()?;
            infinix.auction_length = auction_length;
        }

        emit!(AuctionLengthSet {
            new_auction_length: auction_length
        });
    }

    if let Some(mandate) = mandate {
        check_condition!(
            mandate.len() <= MAX_PADDED_STRING_LENGTH,
            InvalidMandateLength
        );

        {
            let mut infinix = ctx.accounts.infinix.load_mut()?;
            infinix.mandate = FixedSizeString::new(&mandate);
        }
    }

    Ok(())
}
