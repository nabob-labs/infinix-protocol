//! RWA资产authorize指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetAuthorized;

/// RWA资产authorize指令账户上下文
#[derive(Accounts)]
pub struct AuthorizeRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>, // RWA资产账户，需可变
    pub authority: Signer<'info>,             // 当前授权人
    pub new_authority: Pubkey,                // 新授权人公钥
}

/// RWA资产authorize指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - new_authority: 新授权人公钥
pub fn authorize_rwa(ctx: Context<AuthorizeRwa>, new_authority: Pubkey) -> Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    require!(rwa.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    let service = RwaService::new();
    service.authorize(rwa, new_authority)?;
    emit!(AssetAuthorized {
        asset_id: rwa.id,
        old_authority: ctx.accounts.authority.key(),
        new_authority,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 