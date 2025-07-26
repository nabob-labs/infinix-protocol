//! IndexToken资产authorize指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetAuthorized;

/// IndexToken资产authorize指令账户上下文
#[derive(Accounts)]
pub struct AuthorizeIndexToken<'info> {
    #[account(mut)]
    pub index_token: Account<'info, BasketIndexState>, // IndexToken资产账户，需可变
    pub authority: Signer<'info>,                     // 操作人签名者
}

/// IndexToken资产authorize指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - new_authority: 新授权人公钥
pub fn authorize_index_token(ctx: Context<AuthorizeIndexToken>, new_authority: Pubkey) -> Result<()> {
    let index_token = &mut ctx.accounts.index_token;
    require!(index_token.asset_type == AssetType::IndexToken, crate::error::ProgramError::InvalidAssetType);
    let service = IndexTokenService::new();
    service.authorize(index_token, new_authority)?;
    emit!(AssetAuthorized {
        asset_id: index_token.id,
        new_authority,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 