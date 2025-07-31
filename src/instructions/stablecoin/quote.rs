//! Stablecoin资产quote指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetQuoted;

/// Stablecoin资产quote指令账户上下文
#[derive(Accounts)]
pub struct QuoteStablecoin<'info> {
    pub stablecoin: Account<'info, BasketIndexState>, // Stablecoin资产账户
}

/// Stablecoin资产quote指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 询价数量
pub fn quote_stablecoin(ctx: Context<QuoteStablecoin>, amount: u64) -> anchor_lang::Result<()> {
    let stablecoin = &ctx.accounts.stablecoin;
    require!(stablecoin.asset_type == AssetType::Stablecoin, ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    let quote = service.quote(stablecoin, amount)?;
    emit!(AssetQuoted {
        asset_id: stablecoin.id,
        amount,
        quote_value: quote.quote_value,
        price_oracle: quote.price_oracle,
        asset_type: stablecoin.asset_type as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 