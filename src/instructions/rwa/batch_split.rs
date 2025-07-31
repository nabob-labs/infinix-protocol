//!
//! RWA Batch Split Instruction
//! RWA资产批量拆分指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::BatchTradeParams;

/// RWA资产批量拆分指令账户上下文
#[derive(Accounts)]
pub struct BatchSplitRwa<'info> {
    #[account(mut)]
    pub source: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub new_rwas: Vec<Account<'info, BasketIndexState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// RWA资产批量拆分指令实现
pub fn batch_split_rwa(ctx: Context<BatchSplitRwa>, amounts: Vec<u64>, params: BatchTradeParams) -> anchor_lang::Result<()> {
    let source = &mut ctx.accounts.source;
    let new_rwas = &mut ctx.accounts.new_rwas;
    source.validate()?;
    for new_rwa in new_rwas.iter_mut() {
        new_rwa.validate()?;
    }
    require!(source.asset_type == crate::core::types::AssetType::RWA, ProgramError::InvalidAssetType);
    for new_rwa in new_rwas.iter() {
        require!(new_rwa.asset_type == crate::core::types::AssetType::RWA, ProgramError::InvalidAssetType);
    }
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), ProgramError::InvalidAuthority);
    let mut new_refs: Vec<&mut BasketIndexState> = new_rwas.iter_mut().map(|a| a.as_mut()).collect();
    // 业务逻辑：批量拆分
    // TODO: 调用RwaService::batch_split(source, &mut new_refs, &amounts, &params, ctx.accounts.authority.key())
    emit!(RwaBatchSplit {
        source_rwa_id: source.id,
        new_rwa_ids: new_rwas.iter().map(|a| a.id).collect(),
        amounts: amounts.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 