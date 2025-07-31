//! Stablecoin资产swap指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetSwapped;

/// Stablecoin资产swap指令账户上下文
#[derive(Accounts)]
pub struct SwapStablecoin<'info> {
    #[account(mut)]
    pub from_stablecoin: Account<'info, BasketIndexState>, // 转出Stablecoin资产账户，需可变
    #[account(mut)]
    pub to_stablecoin: Account<'info, BasketIndexState>,   // 转入Stablecoin资产账户，需可变
    pub authority: Signer<'info>,                          // 操作人签名者
}

/// Stablecoin资产swap指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - from_amount: 转出数量，类型安全
pub fn swap_stablecoin(ctx: Context<SwapStablecoin>, from_amount: u64) -> anchor_lang::Result<()> {
    let from = &mut ctx.accounts.from_stablecoin;
    let to = &mut ctx.accounts.to_stablecoin;
    require!(from.asset_type == AssetType::Stablecoin && to.asset_type == AssetType::Stablecoin, ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
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