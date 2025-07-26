//! Crypto资产authorize指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetAuthorized;

/// Crypto资产authorize指令账户上下文
#[derive(Accounts)]
pub struct AuthorizeCrypto<'info> {
    #[account(mut)]
    pub crypto: Account<'info, BasketIndexState>, // Crypto资产账户，需可变
    pub authority: Signer<'info>,                // 操作人签名者
}

/// Crypto资产authorize指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - new_authority: 新授权人公钥
pub fn authorize_crypto(ctx: Context<AuthorizeCrypto>, new_authority: Pubkey) -> Result<()> {
    let crypto = &mut ctx.accounts.crypto;
    require!(crypto.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    let service = CryptoService::new();
    service.authorize(crypto, new_authority)?;
    emit!(AssetAuthorized {
        asset_id: crypto.id,
        new_authority,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 