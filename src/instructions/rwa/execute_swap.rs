//!
//! RWA Execute Swap Instruction
//! RWA资产执行swap指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{TradeParams, OracleParams, AssetType};

/// RWA资产执行swap指令账户上下文
#[derive(Accounts)]
pub struct ExecuteSwapRwa<'info> {
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// RWA资产执行swap指令实现
pub fn execute_swap_rwa(ctx: Context<ExecuteSwapRwa>, from_amount: u64, to_amount: u64, params: TradeParams, price_params: OracleParams) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;
    from.validate()?;
    to.validate()?;
    require!(from.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    require!(to.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：调用服务层执行swap
    // TODO: 调用RwaService::execute_swap(from, to, from_amount, to_amount, &params, &price_params, ctx.accounts.authority.key())
    emit!(RwaSwapExecuted {
        from_rwa_id: from.id,
        to_rwa_id: to.id,
        from_amount,
        to_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 