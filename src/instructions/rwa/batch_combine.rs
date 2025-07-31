//!
//! RWA Batch Combine Instruction
//! RWA资产批量合并指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::BatchTradeParams;

/// RWA资产批量合并指令账户上下文
#[derive(Accounts)]
pub struct BatchCombineRwa<'info> {
    #[account(mut)]
    pub target: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub source_rwas: Vec<Account<'info, BasketIndexState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// RWA资产批量合并指令实现
pub fn batch_combine_rwa(ctx: Context<BatchCombineRwa>, params: BatchTradeParams) -> anchor_lang::Result<()> {
    let target = &mut ctx.accounts.target;
    let source_rwas = &mut ctx.accounts.source_rwas;
    target.validate()?;
    for source in source_rwas.iter_mut() {
        source.validate()?;
    }
    require!(target.asset_type == crate::core::types::AssetType::RWA, ProgramError::InvalidAssetType);
    for source in source_rwas.iter() {
        require!(source.asset_type == crate::core::types::AssetType::RWA, ProgramError::InvalidAssetType);
    }
    require_keys_eq!(ctx.accounts.authority.key(), target.authority, ProgramError::InvalidAuthority);
    let mut source_refs: Vec<&mut BasketIndexState> = source_rwas.iter_mut().map(|a| a.as_mut()).collect();
    // 业务逻辑：批量合并
    // TODO: 调用RwaService::batch_combine(target, &mut source_refs, &params, ctx.accounts.authority.key())
    emit!(RwaBatchCombined {
        target_rwa_id: target.id,
        source_rwa_ids: source_rwas.iter().map(|a| a.id).collect(),
        amounts: params.trades.iter().map(|s| s.from_amount).collect(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 