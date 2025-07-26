//!
//! RWA Batch Transfer Instruction
//! RWA资产批量转账指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;

/// RWA资产批量转账指令账户上下文
#[derive(Accounts)]
pub struct BatchTransferRwa<'info> {
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to_rwas: Vec<Account<'info, BasketIndexState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// RWA资产批量转账指令实现
pub fn batch_transfer_rwa(ctx: Context<BatchTransferRwa>, amounts: Vec<u64>) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to_rwas = &mut ctx.accounts.to_rwas;
    from.validate()?;
    for to in to_rwas.iter_mut() {
        to.validate()?;
    }
    require!(from.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    for to in to_rwas.iter() {
        require!(to.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    }
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    let mut to_refs: Vec<&mut BasketIndexState> = to_rwas.iter_mut().map(|a| a.as_mut()).collect();
    // 业务逻辑：批量转账
    // TODO: 调用RwaService::batch_transfer(from, &mut to_refs, &amounts, ctx.accounts.authority.key())
    emit!(RwaBatchTransferred {
        from_rwa_id: from.id,
        to_rwa_ids: to_rwas.iter().map(|a| a.id).collect(),
        amounts: amounts.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 