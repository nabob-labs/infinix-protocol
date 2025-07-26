//! RWA资产mint指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetMinted;

/// RWA资产mint指令账户上下文
#[derive(Accounts)]
pub struct MintRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>, // RWA资产账户，需可变
    pub authority: Signer<'info>, // 操作人签名者
}

/// RWA资产mint指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 增发数量，类型安全
pub fn mint_rwa(ctx: Context<MintRwa>, amount: u64) -> Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    require!(rwa.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    let service = RwaService::new();
    service.mint(rwa, amount)?;
    emit!(AssetMinted {
        basket_id: rwa.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 