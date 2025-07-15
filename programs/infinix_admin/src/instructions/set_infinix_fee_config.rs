use crate::state::{DAOFeeConfig, InfinixFeeConfig};
use crate::utils::InfinixProgram;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};
use shared::check_condition;
use shared::constants::common::ADMIN;
use shared::constants::{
    DAO_FEE_CONFIG_SEEDS, INFINIX_FEE_CONFIG_SEEDS, INFINIX_PROGRAM_ID, INFINIX_SEEDS, MAX_DAO_FEE,
    MAX_FEE_FLOOR,
};
use shared::errors::ErrorCode;

#[derive(Accounts)]
pub struct SetInfinixFeeConfig<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Interface<'info, TokenInterface>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [DAO_FEE_CONFIG_SEEDS],
        bump = dao_fee_config.bump
    )]
    pub dao_fee_config: Account<'info, DAOFeeConfig>,
    #[account(mut)]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut,
        seeds = [INFINIX_SEEDS, infinix_token_mint.key().as_ref()],
        bump,
        seeds::program = INFINIX_PROGRAM_ID,
    )]
    pub infinix: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = admin,
        space = InfinixFeeConfig::SIZE,
        seeds = [INFINIX_FEE_CONFIG_SEEDS, infinix.key().as_ref()],
        bump
    )]
    pub infinix_fee_config: Account<'info, InfinixFeeConfig>,
    #[account(mut)]
    pub infinix_program: UncheckedAccount<'info>,
    #[account(mut)]
    pub fee_recipients: UncheckedAccount<'info>,
    #[account(mut)]
    pub fee_distribution: UncheckedAccount<'info>,
    #[account(mut)]
    pub dao_fee_recipient: UncheckedAccount<'info>,
}

impl SetInfinixFeeConfig<'_> {
    pub fn validate(
        &self,
        scaled_fee_numerator: &Option<u128>,
        scaled_fee_floor: &Option<u128>,
    ) -> Result<()> {
        check_condition!(self.admin.key() == ADMIN, Unauthorized);

        if let Some(scaled_fee_numerator) = scaled_fee_numerator {
            check_condition!(*scaled_fee_numerator <= MAX_DAO_FEE, InvalidFeeNumerator);
        }

        if let Some(scaled_fee_floor) = scaled_fee_floor {
            check_condition!(*scaled_fee_floor <= MAX_FEE_FLOOR, InvalidFeeFloor);
        }

        Ok(())
    }
}

pub fn handler(
    ctx: Context<SetInfinixFeeConfig>,
    scaled_fee_numerator: Option<u128>,
    scaled_fee_floor: Option<u128>,
) -> Result<()> {
    ctx.accounts.validate(&scaled_fee_numerator, &scaled_fee_floor)?;

    InfinixProgram::distribute_fees_cpi(
        &ctx.accounts.infinix_program.to_account_info(),
        &ctx.accounts.rent.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.admin.to_account_info(),
        &ctx.accounts.dao_fee_config.to_account_info(),
        &ctx.accounts.infinix_fee_config.to_account_info(),
        &ctx.accounts.infinix.to_account_info(),
        &ctx.accounts.infinix_token_mint.to_account_info(),
        &ctx.accounts.fee_recipients.to_account_info(),
        &ctx.accounts.fee_distribution.to_account_info(),
        &ctx.accounts.dao_fee_recipient.to_account_info(),
    )?;

    let infinix_fee_config = &mut ctx.accounts.infinix_fee_config;

    InfinixFeeConfig::init_or_update_infinix_fee_config(
       infinix_fee_config,
       &ctx.accounts.dao_fee_config,
       ctx.bumps.infinix_fee_config,
       scaled_fee_numerator,
       scaled_fee_floor,
    )?;

    Ok(())
}