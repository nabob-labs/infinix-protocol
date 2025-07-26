//! Stock资产sell指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetSold;

/// Stock资产sell指令账户上下文
#[derive(Accounts)]
pub struct SellStock<'info> {
    #[account(mut)]
    pub stock: Account<'info, BasketIndexState>, // Stock资产账户，需可变
    pub authority: Signer<'info>,               // 操作人签名者
}

/// Stock资产sell指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 卖出数量，类型安全
pub fn sell_stock(ctx: Context<SellStock>, amount: u64) -> Result<()> {
    let stock = &mut ctx.accounts.stock;
    require!(stock.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    let service = StockService::new();
    service.sell(stock, amount)?;
    emit!(AssetSold {
        basket_id: stock.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 