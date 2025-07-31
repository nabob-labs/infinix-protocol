//! Stablecoin资产sell指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetSold;

/// Stablecoin资产sell指令账户上下文
#[derive(Accounts)]
pub struct SellStablecoin<'info> {
    #[account(mut)]
    pub stablecoin: Account<'info, BasketIndexState>, // Stablecoin资产账户，需可变
    pub seller: Signer<'info>, // 卖出人签名者
}

/// Stablecoin资产sell指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 卖出数量，类型安全
pub fn sell_stablecoin(ctx: Context<SellStablecoin>, amount: u64) -> anchor_lang::Result<()> {
    let stablecoin = &mut ctx.accounts.stablecoin;
    require!(stablecoin.asset_type == AssetType::Stablecoin, ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    service.sell(stablecoin, amount)?;
    emit!(AssetSold {
        basket_id: stablecoin.id,
        amount,
        authority: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 