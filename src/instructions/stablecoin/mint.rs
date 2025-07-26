//! Stablecoin资产mint指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetMinted;

/// Stablecoin资产mint指令账户上下文
#[derive(Accounts)]
pub struct MintStablecoin<'info> {
    #[account(mut)]
    pub stablecoin: Account<'info, BasketIndexState>, // Stablecoin资产账户，需可变
    pub authority: Signer<'info>, // 操作人签名者
}

/// Stablecoin资产mint指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 增发数量，类型安全
pub fn mint_stablecoin(ctx: Context<MintStablecoin>, amount: u64) -> Result<()> {
    let stablecoin = &mut ctx.accounts.stablecoin;
    require!(stablecoin.asset_type == AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    service.mint(stablecoin, amount)?;
    emit!(AssetMinted {
        basket_id: stablecoin.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 