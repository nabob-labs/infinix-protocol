//! Crypto资产buy指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetBought;

/// Crypto资产buy指令账户上下文
#[derive(Accounts)]
pub struct BuyCrypto<'info> {
    #[account(mut)]
    pub crypto: Account<'info, BasketIndexState>, // Crypto资产账户，需可变
    pub authority: Signer<'info>,                // 操作人签名者
}

/// Crypto资产buy指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 购买数量，类型安全
pub fn buy_crypto(ctx: Context<BuyCrypto>, amount: u64) -> Result<()> {
    let crypto = &mut ctx.accounts.crypto;
    require!(crypto.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    let service = CryptoService::new();
    service.buy(crypto, amount)?;
    emit!(AssetBought {
        basket_id: crypto.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 