//! RWA资产batch指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{AssetType, BatchSwapParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::BatchAssetTransferred;

/// RWA资产batch指令账户上下文
#[derive(Accounts)]
pub struct BatchRwa<'info> {
    #[account(mut)]
    pub from_rwa: Account<'info, BasketIndexState>, // 批量转出RWA资产账户，需可变
    #[account(mut)]
    pub to_rwas: Vec<Account<'info, BasketIndexState>>, // 批量转入RWA资产账户，需可变
    pub authority: Signer<'info>, // 操作人签名者
}

/// RWA资产batch指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: BatchSwapParams 批量参数，类型安全
pub fn batch_rwa(ctx: Context<BatchRwa>, params: BatchSwapParams) -> Result<()> {
    let from = &mut ctx.accounts.from_rwa;
    let to_rwas = &mut ctx.accounts.to_rwas;
    require!(from.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    for to in to_rwas.iter() {
        require!(to.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    }
    let service = RwaService::new();
    service.batch_transfer(from, to_rwas, &params, ctx.accounts.authority.key())?;
    emit!(BatchAssetTransferred {
        from_asset_id: from.id,
        to_asset_ids: to_rwas.iter().map(|a| a.id).collect(),
        amounts: params.amounts.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 