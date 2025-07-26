//! Stock资产query指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetQueried;

/// Stock资产query指令账户上下文
#[derive(Accounts)]
pub struct QueryStock<'info> {
    pub stock: Account<'info, BasketIndexState>, // Stock资产账户
}

/// Stock资产query指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
pub fn query_stock(ctx: Context<QueryStock>) -> Result<()> {
    let stock = &ctx.accounts.stock;
    require!(stock.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    let service = StockService::new();
    let info = service.query(stock)?;
    emit!(AssetQueried {
        asset_id: stock.id,
        total_value: info.total_value,
        is_active: info.is_active,
        authority: stock.authority,
        asset_type: stock.asset_type as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 