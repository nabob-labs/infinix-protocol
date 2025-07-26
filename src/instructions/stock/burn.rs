//! Stock资产burn指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetBurned;

/// Stock资产burn指令账户上下文
#[derive(Accounts)]
pub struct BurnStock<'info> {
    #[account(mut)]
    pub stock: Account<'info, BasketIndexState>, // Stock资产账户，需可变
    pub authority: Signer<'info>,               // 操作人签名者
}

/// Stock资产burn指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 销毁数量，类型安全
pub fn burn_stock(ctx: Context<BurnStock>, amount: u64) -> Result<()> {
    let stock = &mut ctx.accounts.stock;
    require!(stock.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    let service = StockService::new();
    service.burn(stock, amount)?;
    emit!(AssetBurned {
        basket_id: stock.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 