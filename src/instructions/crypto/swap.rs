//! Crypto资产swap指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetSwapped;

/// Crypto资产swap指令账户上下文
#[derive(Accounts)]
pub struct SwapCrypto<'info> {
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>, // 转出账户，需可变
    #[account(mut)]
    pub to: Account<'info, BasketIndexState>,   // 转入账户，需可变
    pub authority: Signer<'info>,              // 操作人签名者
}

/// Crypto资产swap指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 兑换数量，类型安全
pub fn swap_crypto(ctx: Context<SwapCrypto>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;
    require!(from.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    require!(to.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    let service = CryptoService::new();
    service.swap(from, to, amount)?;
    emit!(AssetSwapped {
        from_id: from.id,
        to_id: to.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 