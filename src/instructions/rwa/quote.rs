//! RWA资产quote指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{AssetType, PriceParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetQuoted;

/// RWA资产quote指令账户上下文
#[derive(Accounts)]
pub struct QuoteRwa<'info> {
    pub rwa: Account<'info, BasketIndexState>, // RWA资产账户，只读
}

/// RWA资产quote指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - price_params: 价格参数，类型安全
pub fn quote_rwa(ctx: Context<QuoteRwa>, price_params: PriceParams) -> Result<u64> {
    let rwa = &ctx.accounts.rwa;
    require!(rwa.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    let service = RwaService::new();
    let price = service.quote(rwa, price_params)?;
    emit!(AssetQuoted {
        asset_id: rwa.id,
        price,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(price)
} 