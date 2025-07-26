//! Stock资产quote指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetQuoted;

/// Stock资产quote指令账户上下文
#[derive(Accounts)]
pub struct QuoteStock<'info> {
    pub stock: Account<'info, BasketIndexState>, // Stock资产账户
}

/// Stock资产quote指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 询价数量
pub fn quote_stock(ctx: Context<QuoteStock>, amount: u64) -> Result<()> {
    let stock = &ctx.accounts.stock;
    require!(stock.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    let service = StockService::new();
    let quote = service.quote(stock, amount)?;
    emit!(AssetQuoted {
        asset_id: stock.id,
        amount,
        quote_value: quote.quote_value,
        price_oracle: quote.price_oracle,
        asset_type: stock.asset_type as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 