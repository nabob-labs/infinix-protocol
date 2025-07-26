//! ETF资产quote指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetQuoted;

/// ETF资产quote指令账户上下文
#[derive(Accounts)]
pub struct QuoteEtf<'info> {
    pub etf: Account<'info, BasketIndexState>, // ETF资产账户
}

/// ETF资产quote指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 询价数量
pub fn quote_etf(ctx: Context<QuoteEtf>, amount: u64) -> Result<()> {
    let etf = &ctx.accounts.etf;
    require!(etf.asset_type == AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    let service = EtfService::new();
    let quote = service.quote(etf, amount)?;
    emit!(AssetQuoted {
        asset_id: etf.id,
        amount,
        quote_value: quote.quote_value,
        price_oracle: quote.price_oracle,
        asset_type: etf.asset_type as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 