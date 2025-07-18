use crate::events::RebalanceStarted;
use crate::state::Rebalance;
use crate::state::{Actor, Infinix};
use crate::utils::structs::{InfinixStatus, Role};
use crate::utils::RebalancePriceAndLimits;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use infinix_admin::state::DAOFeeConfig;
use infinix_admin::ID as INFINIX_ADMIN_PROGRAM_ID;
use shared::constants::{DAO_FEE_CONFIG_SEEDS, INFINIX_FEE_CONFIG_SEEDS, REBALANCE_SEEDS};
use shared::utils::TokenUtil;
use shared::{check_condition, constants::ACTOR_SEEDS, errors::ErrorCode};

#[derive(Accounts)]
#[instruction()]
pub struct StartRebalance<'info> {
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
        init_if_needed,
        payer = rebalance_manager,
        space = Rebalance::SIZE,
        seeds = [REBALANCE_SEEDS, infinix.key().as_ref()],
        bump
    )]
    pub rebalance: AccountLoader<'info, Rebalance>,

    #[account(
        seeds = [DAO_FEE_CONFIG_SEEDS],
        bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub dao_fee_config: Account<'info, DAOFeeConfig>,

    #[account(
        seeds = [INFINIX_FEE_CONFIG_SEEDS, infinix.key().as_ref()],
        bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub infinix_fee_config: UncheckedAccount<'info>,

    #[account(mut)]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,
    // remaining accounts:
    // - token mints for rebalance
}

impl StartRebalance<'_> {
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

        check_condition!(
            self.infinix_token_mint.key() == infinix.infinix_token_mint,
            InvalidInfinixTokenMint
        );

        Ok(())
    }
}

pub fn handler(
    ctx: Context<StartRebalance>,
    auction_launcher_window: u64,
    ttl: u64,
    prices_and_limits: Vec<RebalancePriceAndLimits>,
    all_rebalance_details_added: bool,
) -> Result<()> {
    let infinix_key = ctx.accounts.infinix.key();
    let infinix = &mut ctx.accounts.infinix.load_mut()?;
    let mints = ctx.remaining_accounts;

    ctx.accounts.validate(infinix, mints)?;

    let current_time = Clock::get()?.unix_timestamp;
    {
        // Poke infinix
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
    }

    // Initialize rebalance account if needed
    Rebalance::process_init_if_needed(
        &mut ctx.accounts.rebalance,
        ctx.bumps.rebalance,
        &infinix_key,
    )?;

    let rebalance_res = &mut ctx.accounts.rebalance.load_mut();
    let rebalance = match rebalance_res {
        Ok(rebalance) => rebalance,
        Err(_) => &mut ctx.accounts.rebalance.load_init()?,
    };

    let current_time = current_time as u64;
    rebalance.start_rebalance(
        current_time,
        auction_launcher_window,
        ttl,
        mints,
        prices_and_limits,
        all_rebalance_details_added,
    )?;

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
