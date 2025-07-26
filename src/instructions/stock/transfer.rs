//! Stock资产transfer指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetTransferred;

/// Stock资产transfer指令账户上下文
#[derive(Accounts)]
pub struct TransferStock<'info> {
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>, // 转出账户，需可变
    #[account(mut)]
    pub to: Account<'info, BasketIndexState>,   // 转入账户，需可变
    pub authority: Signer<'info>,              // 操作人签名者
}

/// Stock资产transfer指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 转账数量，类型安全
pub fn transfer_stock(ctx: Context<TransferStock>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;
    require!(from.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    require!(to.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    let service = StockService::new();
    service.transfer(from, to, amount)?;
    emit!(AssetTransferred {
        from_id: from.id,
        to_id: to.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 