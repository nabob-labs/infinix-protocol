//! Stablecoin资产combine指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetCombined;

/// Stablecoin资产combine指令账户上下文
#[derive(Accounts)]
pub struct CombineStablecoin<'info> {
    #[account(mut)]
    pub target_stablecoin: Account<'info, BasketIndexState>, // 目标Stablecoin资产账户，需可变
    #[account(mut)]
    pub source_stablecoin: Account<'info, BasketIndexState>, // 源Stablecoin资产账户，需可变
    pub authority: Signer<'info>,                           // 操作人签名者
}

/// Stablecoin资产combine指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 合并数量，类型安全
pub fn combine_stablecoin(ctx: Context<CombineStablecoin>, amount: u64) -> anchor_lang::Result<()> {
    let target = &mut ctx.accounts.target_stablecoin;
    let source = &mut ctx.accounts.source_stablecoin;
    require!(target.asset_type == AssetType::Stablecoin && source.asset_type == AssetType::Stablecoin, ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
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