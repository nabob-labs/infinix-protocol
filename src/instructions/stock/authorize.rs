//! Stock资产authorize指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetAuthorized;

/// Stock资产authorize指令账户上下文
#[derive(Accounts)]
pub struct AuthorizeStock<'info> {
    #[account(mut)]
    pub stock: Account<'info, BasketIndexState>, // Stock资产账户，需可变
    pub authority: Signer<'info>,               // 操作人签名者
}

/// Stock资产authorize指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - new_authority: 新授权人公钥
pub fn authorize_stock(ctx: Context<AuthorizeStock>, new_authority: Pubkey) -> Result<()> {
    let stock = &mut ctx.accounts.stock;
    require!(stock.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    let service = StockService::new();
    service.authorize(stock, new_authority)?;
    emit!(AssetAuthorized {
        asset_id: stock.id,
        new_authority,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 