//!
//! RWA Strategy Trade Instruction
//! RWA资产策略交易指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{StrategyParams, TradeParams, OracleParams, AssetType};

/// RWA资产策略交易指令账户上下文
#[derive(Accounts)]
pub struct StrategyTradeRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// RWA资产策略交易指令实现
pub fn strategy_trade_rwa(ctx: Context<StrategyTradeRwa>, params: { strategy: StrategyParams, swap_params: Option<TradeParams>, price_params: Option<OracleParams>, exec_params: Option<TradeParams> }) -> Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    rwa.validate()?;
    require!(rwa.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(rwa.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：策略交易
    // TODO: 调用RwaService::strategy_trade(rwa, &params, ctx.accounts.authority.key())
    emit!(RwaStrategyTraded {
        rwa_id: rwa.id,
        strategy: params.strategy.strategy_name.to_string(),
        params: params.strategy.params.to_vec(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 