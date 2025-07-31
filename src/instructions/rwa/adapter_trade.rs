//!
//! RWA Adapter Trade Instruction
//! RWA资产适配器交易指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::*;

/// RWA资产适配器交易指令账户上下文
#[derive(Accounts)]
pub struct AdapterTradeRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// RWA资产适配器交易指令实现
pub fn adapter_trade_rwa(ctx: Context<AdapterTradeRwa>, adapter_params: AdapterParams, trade_params: TradeParams) -> anchor_lang::Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    rwa.validate()?;
    require!(rwa.asset_type == AssetType::RWA, ProgramError::InvalidAssetType);
    require_keys_eq!(rwa.authority, ctx.accounts.authority.key(), ProgramError::InvalidAuthority);
    // 业务逻辑：适配器交易
    // TODO: 调用RwaService::adapter_trade(rwa, &adapter_params, &trade_params, ctx.accounts.authority.key())
    emit!(RwaAdapterTraded {
        rwa_id: rwa.id,
        adapter_name: adapter_params.adapter_name.to_string(),
        params: adapter_params.params.to_vec(),
        trade_params: trade_params,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 