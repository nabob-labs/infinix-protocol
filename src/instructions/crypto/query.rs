//! Crypto资产query指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetQueried;

/// Crypto资产query指令账户上下文
#[derive(Accounts)]
pub struct QueryCrypto<'info> {
    pub crypto: Account<'info, BasketIndexState>, // Crypto资产账户
}

/// Crypto资产query指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
pub fn query_crypto(ctx: Context<QueryCrypto>) -> Result<()> {
    let crypto = &ctx.accounts.crypto;
    require!(crypto.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    let service = CryptoService::new();
    let info = service.query(crypto)?;
    emit!(AssetQueried {
        asset_id: crypto.id,
        total_value: info.total_value,
        is_active: info.is_active,
        authority: crypto.authority,
        asset_type: crypto.asset_type as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 