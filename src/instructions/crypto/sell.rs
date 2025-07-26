//! Crypto资产sell指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetSold;

/// Crypto资产sell指令账户上下文
#[derive(Accounts)]
pub struct SellCrypto<'info> {
    #[account(mut)]
    pub crypto: Account<'info, BasketIndexState>, // Crypto资产账户，需可变
    pub authority: Signer<'info>,                // 操作人签名者
}

/// Crypto资产sell指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 卖出数量，类型安全
pub fn sell_crypto(ctx: Context<SellCrypto>, amount: u64) -> Result<()> {
    let crypto = &mut ctx.accounts.crypto;
    require!(crypto.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    let service = CryptoService::new();
    service.sell(crypto, amount)?;
    emit!(AssetSold {
        basket_id: crypto.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 