use crate::utils::structs::InfinixStatus;
use crate::ID;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token::ID as TOKEN_PROGRAM_ID;
use anchor_spl::token_2022::ID as TOKEN_2022_PROGRAM_ID;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use anchor_spl::{token_2022, token_interface};
use infinix_admin::state::DAOFeeConfig;
use infinix_admin::ID as INFINIX_ADMIN_PROGRAM_ID;
use shared::check_condition;
use shared::constants::{
    D9_U128, DAO_FEE_CONFIG_SEEDS, FEE_DISTRIBUTION_SEEDS, FEE_RECIPIENTS_SEEDS,
    INFINIX_FEE_CONFIG_SEEDS, INFINIX_SEEDS,
};
use shared::errors::ErrorCode;
use shared::utils::{Decimal, Rounding};

use crate::events::ProtocolFeePaid;
use crate::state::{FeeDistribution, FeeRecipients, Infinix};

#[derive(Accounts)]
#[instruction(index: u64)]
pub struct DistributeFees<'info> {
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: seeds validated in validate function
    #[account()]
    pub dao_fee_config: Account<'info, DAOFeeConfig>,

    /// CHECK: Could be empty or could be set, if set we use that one, else we use dao fee config, seeds validated in validate function
    #[account()]
    pub infinix_fee_config: UncheckedAccount<'info>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut)]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: seeds validated in validate function
    #[account(mut)]
    pub fee_recipients: AccountLoader<'info, FeeRecipients>,

    #[account(
        init,
        payer = user,
        space = FeeDistribution::SIZE,
        seeds = [FEE_DISTRIBUTION_SEEDS, infinix.key().as_ref(), index.to_le_bytes().as_slice()],
        bump,
    )]
    pub fee_distribution: AccountLoader<'info, FeeDistribution>,

    #[account(mut)]
    pub dao_fee_recipient: Box<InterfaceAccount<'info, TokenAccount>>,
}

pub fn validate<'info>(
    infinix: &AccountLoader<'info, Infinix>,
    fee_recipients: &FeeRecipients,
    infinix_token_mint: &InterfaceAccount<'info, Mint>,
    dao_fee_config: &Account<'info, DAOFeeConfig>,
    infinix_fee_config: &AccountInfo<'info>,
    fee_recipients_account: &AccountInfo<'info>,
    fee_distribution_account: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    index: u64,
) -> Result<()> {
    let loaded_infinix = infinix.load()?;

    loaded_infinix.validate_infinix(
        &infinix.key(),
        None,
        None,
        Some(vec![InfinixStatus::Initialized, InfinixStatus::Killed]),
    )?;

    check_condition!(
        fee_recipients.distribution_index + 1 == index,
        InvalidDistributionIndex
    );

    check_condition!(
        infinix_token_mint.key() == loaded_infinix.infinix_token_mint,
        InvalidInfinixTokenMint
    );

    check_condition!(
        *infinix_token_mint.to_account_info().owner == token_program.key(),
        InvalidTokenMintProgram
    );

    let infinix_key = infinix.key();

    // Validate dao_fee_config PDA
    let (expected_dao_fee_config, _) =
        Pubkey::find_program_address(&[DAO_FEE_CONFIG_SEEDS], &INFINIX_ADMIN_PROGRAM_ID);

    check_condition!(dao_fee_config.key() == expected_dao_fee_config, InvalidPda);

    // Validate folio_fee_config PDA
    let (expected_infinix_fee_config, _) = Pubkey::find_program_address(
        &[INFINIX_FEE_CONFIG_SEEDS, infinix_key.as_ref()],
        &INFINIX_ADMIN_PROGRAM_ID,
    );
    check_condition!(
        infinix_fee_config.key() == expected_infinix_fee_config,
        InvalidPda
    );

    // Validate fee_recipients PDA
    let (expected_fee_recipients, _) =
        Pubkey::find_program_address(&[FEE_RECIPIENTS_SEEDS, infinix_key.as_ref()], &ID);
    check_condition!(
        fee_recipients_account.key() == expected_fee_recipients,
        InvalidPda
    );

    // Validate fee_distribution PDA
    let (expected_fee_distribution, _) = Pubkey::find_program_address(
        &[
            FEE_DISTRIBUTION_SEEDS,
            infinix_key.as_ref(),
            index.to_le_bytes().as_slice(),
        ],
        &ID,
    );
    check_condition!(
        fee_distribution_account.key() == expected_fee_distribution,
        InvalidFeeDistribution
    );

    // Validate the token program
    check_condition!(
        [TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID].contains(&token_program.key()),
        InvalidProgram
    );

    Ok(())
}

pub fn distribute_fees<'info>(
    token_program: &AccountInfo<'info>,
    user: &AccountInfo<'info>,
    dao_fee_config: &Account<'info, DAOFeeConfig>,
    infinix_fee_config: &AccountInfo<'info>,
    infinix: &AccountLoader<'info, Infinix>,
    infinix_token_mint: &InterfaceAccount<'info, Mint>,
    fee_recipients: &AccountLoader<'info, FeeRecipients>,
    fee_distribution: &AccountLoader<'info, FeeDistribution>,
    dao_fee_recipient: &AccountInfo<'info>,
    index: u64,
) -> Result<()> {
    {
        let fee_recipients_data = fee_recipients.load()?;

        validate(
            infinix,
            &fee_recipients_data,
            infinix_token_mint,
            dao_fee_config,
            infinix_fee_config,
            &fee_recipients.to_account_info(),
            &fee_distribution.to_account_info(),
            token_program,
            index,
        )?;

        let infinix = &mut infinix.load_mut()?;

        let fee_details = dao_fee_config.get_fee_details(infinix_fee_config)?;

        // Validate token account for the DAO fee recipient
        check_condition!(
            dao_fee_recipient.key()
                == get_associated_token_address_with_program_id(
                    &fee_details.fee_recipient,
                    &infinix_token_mint.key(),
                    &token_program.key(),
                ),
            InvalidDaoFeeRecipient
        );

        // Update pending fees by poking to get latest fees
        let current_time = Clock::get()?.unix_timestamp;
        infinix.poke(
            infinix_token_mint.supply,
            current_time,
            fee_details.scaled_fee_numerator,
            fee_details.scaled_fee_denominator,
            fee_details.scaled_fee_floor,
        )?;
    }

    // Mint pending fees to dao recipient
    let mut raw_dao_pending_fee_shares: u64;

    let scaled_fee_recipients_pending_fee_shares_minus_dust: u128;

    let has_fee_recipients: bool;

    {
        let infinix_key = infinix.key();
        let loaded_infinix = infinix.load()?;
        let fee_recipients = fee_recipients.load()?;
        let token_mint_key = infinix_token_mint.key();

        has_fee_recipients = !fee_recipients.is_empty();

        // We scale down as token units and bring back in D9, to get the amount
        // minus the dust that we can split
        let raw_fee_recipients_pending_fee_shares: u64 =
            Decimal::from_scaled(loaded_infinix.fee_recipients_pending_fee_shares)
                .to_token_amount(Rounding::Floor)?
                .0;

        scaled_fee_recipients_pending_fee_shares_minus_dust =
            (raw_fee_recipients_pending_fee_shares as u128)
                .checked_mul(D9_U128)
                .ok_or(ErrorCode::MathOverflow)?;

        raw_dao_pending_fee_shares = Decimal::from_scaled(loaded_infinix.dao_pending_fee_shares)
            .to_token_amount(Rounding::Floor)?
            .0;

        let bump = loaded_infinix.bump;
        let signer_seeds = &[INFINIX_SEEDS, token_mint_key.as_ref(), &[bump]];

        let cpi_accounts = token_2022::MintTo {
            mint: infinix_token_mint.to_account_info(),
            to: dao_fee_recipient.to_account_info(),
            authority: infinix.to_account_info(),
        };

        if !has_fee_recipients {
            // If there are no fee recipients, the DAO gets all the fees
            raw_dao_pending_fee_shares = raw_dao_pending_fee_shares
                .checked_add(raw_fee_recipients_pending_fee_shares)
                .ok_or(ErrorCode::MathOverflow)?;
        }

        token_interface::mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                cpi_accounts,
                &[signer_seeds],
            ),
            raw_dao_pending_fee_shares,
        )?;

        // Create new fee distribution for other recipients if there are any
        if has_fee_recipients {
            let fee_distribution_loaded = &mut fee_distribution.load_init()?;

            let (fee_distribution_derived_key, bump) = Pubkey::find_program_address(
                &[
                    FEE_DISTRIBUTION_SEEDS,
                    infinix_key.as_ref(),
                    index.to_le_bytes().as_slice(),
                ],
                &ID,
            );

            // Make the the derived key is the right one
            check_condition!(
                fee_distribution_derived_key == fee_distribution.key(),
                InvalidFeeDistribution
            );

            fee_distribution_loaded.bump = bump;
            fee_distribution_loaded.index = index;
            fee_distribution_loaded.infinix = infinix_key;
            fee_distribution_loaded.cranker = user.key();
            fee_distribution_loaded.amount_to_distribute =
                scaled_fee_recipients_pending_fee_shares_minus_dust;
            fee_distribution_loaded.fee_recipients_state = fee_recipients.fee_recipients;
        } else {
            // We close it if there are no fee recipients
            fee_distribution.close(user.to_account_info())?;
        }

        emit!(ProtocolFeePaid {
            recipient: dao_fee_recipient.key(),
            amount: raw_dao_pending_fee_shares,
        });
    }

    // Update infinix pending fees based on what was distributed
    {
        let infinix = &mut infinix.load_mut()?;

        if has_fee_recipients {
            infinix.dao_pending_fee_shares = infinix
                .dao_pending_fee_shares
                .checked_sub(
                    (raw_dao_pending_fee_shares as u128)
                        // Got to multiply back in D9 since we track with extra precision
                        .checked_mul(D9_U128)
                        .ok_or(ErrorCode::MathOverflow)?,
                )
                .ok_or(ErrorCode::MathOverflow)?;
        } else {
            // In case of no fee recipients, it is possible that `raw_dao_pending_fee_shares` is higher than the `dao_pending_fee_shares`
            infinix.dao_pending_fee_shares = infinix.dao_pending_fee_shares.saturating_sub(
                (raw_dao_pending_fee_shares as u128)
                    // Got to multiply back in D9 since we track with extra precision
                    .checked_mul(D9_U128)
                    .ok_or(ErrorCode::MathOverflow)?,
            );
        }

        // Still remove from the fee recipient pending shares even if there are no fee recipients
        // as it's given to the DAO
        infinix.fee_recipients_pending_fee_shares = infinix
            .fee_recipients_pending_fee_shares
            .checked_sub(scaled_fee_recipients_pending_fee_shares_minus_dust)
            .unwrap();

        if has_fee_recipients {
            // Add the fees to track total supply, including pending shares
            infinix.fee_recipients_pending_fee_shares_to_be_minted = infinix
                .fee_recipients_pending_fee_shares_to_be_minted
                .checked_add(scaled_fee_recipients_pending_fee_shares_minus_dust)
                .ok_or(ErrorCode::MathOverflow)?;
        }

        let fee_recipients = &mut fee_recipients.load_mut()?;
        fee_recipients.distribution_index = index;
    }

    Ok(())
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, DistributeFees<'info>>,
    index: u64,
) -> Result<()> {
    distribute_fees(
        &ctx.accounts.token_program,
        &ctx.accounts.user,
        &ctx.accounts.dao_fee_config,
        &ctx.accounts.infinix_fee_config,
        &ctx.accounts.infinix,
        &ctx.accounts.infinix_token_mint,
        &ctx.accounts.fee_recipients,
        &ctx.accounts.fee_distribution,
        &ctx.accounts.dao_fee_recipient.to_account_info(),
        index,
    )?;

    Ok(())
}
