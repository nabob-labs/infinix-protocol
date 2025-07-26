//! RWA资产sell指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetSold;

/// RWA资产sell指令账户上下文
#[derive(Accounts)]
pub struct SellRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>, // RWA资产账户，需可变
    pub seller: Signer<'info>, // 卖出人签名者
}

/// RWA资产sell指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 卖出数量，类型安全
pub fn sell_rwa(ctx: Context<SellRwa>, amount: u64) -> Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    require!(rwa.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    let service = RwaService::new();
    service.sell(rwa, amount)?;
    emit!(AssetSold {
        basket_id: rwa.id,
        amount,
        authority: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 