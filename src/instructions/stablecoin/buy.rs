//! Stablecoin资产buy指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetBought;

/// Stablecoin资产buy指令账户上下文
#[derive(Accounts)]
pub struct BuyStablecoin<'info> {
    #[account(mut)]
    pub stablecoin: Account<'info, BasketIndexState>, // Stablecoin资产账户，需可变
    pub buyer: Signer<'info>, // 买入人签名者
}

/// Stablecoin资产buy指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 买入数量，类型安全
pub fn buy_stablecoin(ctx: Context<BuyStablecoin>, amount: u64) -> anchor_lang::Result<()> {
    let stablecoin = &mut ctx.accounts.stablecoin;
    require!(stablecoin.asset_type == AssetType::Stablecoin, ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    service.buy(stablecoin, amount)?;
    emit!(AssetBought {
        basket_id: stablecoin.id,
        amount,
        authority: ctx.accounts.buyer.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 