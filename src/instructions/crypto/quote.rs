//! Crypto资产quote指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetQuoted;

/// Crypto资产quote指令账户上下文
#[derive(Accounts)]
pub struct QuoteCrypto<'info> {
    pub crypto: Account<'info, BasketIndexState>, // Crypto资产账户
}

/// Crypto资产quote指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 询价数量
pub fn quote_crypto(ctx: Context<QuoteCrypto>, amount: u64) -> Result<()> {
    let crypto = &ctx.accounts.crypto;
    require!(crypto.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    let service = CryptoService::new();
    let quote = service.quote(crypto, amount)?;
    emit!(AssetQuoted {
        asset_id: crypto.id,
        amount,
        quote_value: quote.quote_value,
        price_oracle: quote.price_oracle,
        asset_type: crypto.asset_type as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 