//! Crypto资产split指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetSplit;

/// Crypto资产split指令账户上下文
#[derive(Accounts)]
pub struct SplitCrypto<'info> {
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>, // 被拆分账户，需可变
    #[account(mut)]
    pub to: Account<'info, BasketIndexState>,   // 新账户，需可变
    pub authority: Signer<'info>,              // 操作人签名者
}

/// Crypto资产split指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 拆分数量，类型安全
pub fn split_crypto(ctx: Context<SplitCrypto>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;
    require!(from.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    require!(to.asset_type == AssetType::Crypto, crate::error::ProgramError::InvalidAssetType);
    let service = CryptoService::new();
    service.split(from, to, amount)?;
    emit!(AssetSplit {
        from_id: from.id,
        to_id: to.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 