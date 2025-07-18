use crate::events::RebalanceStarted;
use crate::state::Rebalance;
use crate::state::{Actor, Infinix};
use crate::utils::structs::{InfinixStatus, Role};
use crate::utils::RebalancePriceAndLimits;
use anchor_lang::prelude::*;
use shared::constants::REBALANCE_SEEDS;
use shared::utils::TokenUtil;
use shared::{check_condition, constants::ACTOR_SEEDS, errors::ErrorCode};

#[derive(Accounts)]
#[instruction()]
pub struct AddRebalanceDetails<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub rebalance_manager: Signer<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, rebalance_manager.key().as_ref(), infinix.key().as_ref()],
        bump = actor.bump,
    )]
    pub actor: Account<'info, Actor>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(
        mut,
        seeds = [REBALANCE_SEEDS, infinix.key().as_ref()],
        bump = rebalance.load()?.bump,
    )]
    pub rebalance: AccountLoader<'info, Rebalance>,
    // remaining accounts:
    // - token mints for rebalance
}

impl AddRebalanceDetails<'_> {
    pub fn validate(&self, infinix: &Infinix, mints: &[AccountInfo]) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            Some(&self.actor),
            Some(vec![Role::RebalanceManager]),
            Some(vec![InfinixStatus::Initialized]),
        )?;

        for mint in mints {
            // Validate that the buy mint is a supported SPL token (can only check mint here, will check token account in the bid)
            check_condition!(
                TokenUtil::is_supported_spl_token(Some(mint), None)?,
                UnsupportedSPLToken
            );
        }

        Ok(())
    }
}

pub fn handler(
    ctx: Context<AddRebalanceDetails>,
    prices_and_limits: Vec<RebalancePriceAndLimits>,
    all_rebalance_details_added: bool,
) -> Result<()> {
    let infinix = &ctx.accounts.infinix.load()?;
    let mints = ctx.remaining_accounts;

    let rebalance = &mut ctx.accounts.rebalance.load_mut()?;

    ctx.accounts.validate(infinix, mints)?;

    rebalance.add_rebalance_details(mints, prices_and_limits, all_rebalance_details_added)?;

    if all_rebalance_details_added {
        emit!(RebalanceStarted {
            nonce: rebalance.nonce,
            infinix: rebalance.infinix,
            started_at: rebalance.started_at,
            restricted_until: rebalance.restricted_until,
            available_until: rebalance.available_until,
            details: rebalance.details
        });
    }
    Ok(())
}
