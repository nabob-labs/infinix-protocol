//! Stock资产batch指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetBatchProcessed;

/// Stock资产batch指令账户上下文
#[derive(Accounts)]
pub struct BatchStock<'info> {
    #[account(mut)]
    pub stock: Account<'info, BasketIndexState>, // Stock资产账户，需可变
    pub authority: Signer<'info>,               // 操作人签名者
}

/// Stock资产batch指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - actions: 批量操作类型与参数数组
pub fn batch_stock(ctx: Context<BatchStock>, actions: Vec<StockBatchAction>) -> Result<()> {
    let stock = &mut ctx.accounts.stock;
    require!(stock.asset_type == AssetType::Stock, crate::error::ProgramError::InvalidAssetType);
    let service = StockService::new();
    service.batch(stock, &actions)?;
    emit!(AssetBatchProcessed {
        asset_id: stock.id,
        actions_count: actions.len() as u64,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

/// 批量操作类型定义
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StockBatchAction {
    pub action_type: u8, // 0:mint, 1:burn, 2:transfer, 3:freeze, 4:unfreeze, ...
    pub amount: u64,
    pub target: Option<Pubkey>, // 目标账户（如转账、授权等）
} 