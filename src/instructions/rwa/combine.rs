//! RWA资产combine指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetCombined;

/// RWA资产combine指令账户上下文
#[derive(Accounts)]
pub struct CombineRwa<'info> {
    #[account(mut)]
    pub target_rwa: Account<'info, BasketIndexState>, // 目标RWA资产账户，需可变
    #[account(mut)]
    pub source_rwa: Account<'info, BasketIndexState>, // 源RWA资产账户，需可变
    pub authority: Signer<'info>,                    // 操作人签名者
}

/// RWA资产combine指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 合并数量，类型安全
pub fn combine_rwa(ctx: Context<CombineRwa>, amount: u64) -> Result<()> {
    let target = &mut ctx.accounts.target_rwa;
    let source = &mut ctx.accounts.source_rwa;
    require!(target.asset_type == AssetType::RWA && source.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    let service = RwaService::new();
    service.combine(target, source, amount)?;
    emit!(AssetCombined {
        target_asset_id: target.id,
        source_asset_id: source.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 