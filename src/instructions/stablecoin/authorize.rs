//! Stablecoin资产authorize指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetAuthorized;

/// Stablecoin资产authorize指令账户上下文
#[derive(Accounts)]
pub struct AuthorizeStablecoin<'info> {
    #[account(mut)]
    pub stablecoin: Account<'info, BasketIndexState>, // Stablecoin资产账户，需可变
    pub authority: Signer<'info>,                    // 操作人签名者
}

/// Stablecoin资产authorize指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - new_authority: 新授权人公钥
pub fn authorize_stablecoin(ctx: Context<AuthorizeStablecoin>, new_authority: Pubkey) -> Result<()> {
    let stablecoin = &mut ctx.accounts.stablecoin;
    require!(stablecoin.asset_type == AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    service.authorize(stablecoin, new_authority)?;
    emit!(AssetAuthorized {
        asset_id: stablecoin.id,
        new_authority,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 