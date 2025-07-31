//!
//! RWA Execute Combine Instruction
//! RWA资产执行合并指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::*;

/// RWA资产执行合并指令账户上下文
#[derive(Accounts)]
pub struct ExecuteCombineRwa<'info> {
    #[account(mut)]
    pub target: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub source: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// RWA资产执行合并指令实现
pub fn execute_combine_rwa(ctx: Context<ExecuteCombineRwa>, amount: u64, params: TradeParams) -> anchor_lang::Result<()> {
    let target = &mut ctx.accounts.target;
    let source = &mut ctx.accounts.source;
    target.validate()?;
    source.validate()?;
    require!(target.asset_type == AssetType::RWA, ProgramError::InvalidAssetType);
    require!(source.asset_type == AssetType::RWA, ProgramError::InvalidAssetType);
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), ProgramError::InvalidAuthority);
    // 业务逻辑：调用服务层执行合并
    // TODO: 调用RwaService::execute_combine(target, source, amount, &params, ctx.accounts.authority.key())
    emit!(RwaCombineExecuted {
        target_rwa_id: target.id,
        source_rwa_id: source.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 