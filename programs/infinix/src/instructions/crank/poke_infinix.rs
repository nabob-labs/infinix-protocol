use crate::utils::structs::InfinixStatus;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use infinix_admin::state::DAOFeeConfig;
use shared::check_condition;
use shared::constants::{DAO_FEE_CONFIG_SEEDS, INFINIX_FEE_CONFIG_SEEDS};
use shared::errors::ErrorCode;

use crate::state::Infinix;
use infinix_admin::ID as INFINIX_ADMIN_PROGRAM_ID;

#[derive(Accounts)]
pub struct PokeInfinix<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [DAO_FEE_CONFIG_SEEDS],
        bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub dao_fee_config: Account<'info, DAOFeeConfig>,

    /// CHECK: Could be empty or could be set, if set we use that one, else we use dao fee config
    #[account(
            seeds = [INFINIX_FEE_CONFIG_SEEDS, infinix.key().as_ref()],
            bump,
            seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub infinix_fee_config: UncheckedAccount<'info>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut)]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl PokeInfinix<'_> {
    pub fn validate(&self, infinix: &Infinix) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            None,
            None,
            Some(vec![InfinixStatus::Initialized, InfinixStatus::Killed]),
        )?;

        check_condition!(
            self.infinix_token_mint.key() == infinix.infinix_token_mint,
            InvalidInfinixTokenMint
        );

        Ok(())
    }
}

pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info, PokeInfinix<'info>>) -> Result<()> {
    let infinix = &mut ctx.accounts.infinix.load_mut()?;

    ctx.accounts.validate(infinix)?;

    let current_time = Clock::get()?.unix_timestamp;

    let fee_details = ctx
        .accounts
        .dao_fee_config
        .get_fee_details(&ctx.accounts.infinix_fee_config)?;

    infinix.poke(
        ctx.accounts.infinix_token_mint.supply,
        current_time,
        fee_details.scaled_fee_numerator,
        fee_details.scaled_fee_denominator,
        fee_details.scaled_fee_floor,
    )?;

    Ok(())
}
