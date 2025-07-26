//! Stock资产buy指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetBought;

/// Stock资产buy指令账户上下文
#[derive(Accounts)]
pub struct BuyStock<'info> {
    #[account(mut)]
    pub stock: Account<'info, BasketIndexState>, // Stock资产账户，需可变
    pub authority: Signer<'info>,               // 操作人签名者
}

/// Stock资产buy指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 购买数量，类型安全
pub fn buy_stock(ctx: Context<BuyStock>, amount: u64) -> Result<()> {
    let stock = &mut ctx.accounts.stock;
    require!(stock.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    let service = StockService::new();
    service.buy(stock, amount)?;
    emit!(AssetBought {
        basket_id: stock.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 