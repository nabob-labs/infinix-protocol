//! RWA资产swap指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetSwapped;

/// RWA资产swap指令账户上下文
#[derive(Accounts)]
pub struct SwapRwa<'info> {
    #[account(mut)]
    pub from_rwa: Account<'info, BasketIndexState>, // 转出RWA资产账户，需可变
    #[account(mut)]
    pub to_rwa: Account<'info, BasketIndexState>,   // 转入RWA资产账户，需可变
    pub authority: Signer<'info>,                   // 操作人签名者
}

/// RWA资产swap指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - from_amount: 转出数量，类型安全
pub fn swap_rwa(ctx: Context<SwapRwa>, from_amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from_rwa;
    let to = &mut ctx.accounts.to_rwa;
    require!(from.asset_type == AssetType::RWA && to.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    let service = RwaService::new();
    service.swap(from, to, from_amount)?;
    emit!(AssetSwapped {
        from_asset_id: from.id,
        to_asset_id: to.id,
        from_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 