//! IndexToken资产freeze指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetFrozen;

/// IndexToken资产freeze指令账户上下文
#[derive(Accounts)]
pub struct FreezeIndexToken<'info> {
    #[account(mut)]
    pub index_token: Account<'info, BasketIndexState>, // IndexToken资产账户，需可变
    pub authority: Signer<'info>,                     // 操作人签名者
}

/// IndexToken资产freeze指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
pub fn freeze_index_token(ctx: Context<FreezeIndexToken>) -> Result<()> {
    let index_token = &mut ctx.accounts.index_token;
    require!(index_token.asset_type == AssetType::IndexToken, crate::error::ProgramError::InvalidAssetType);
    let service = IndexTokenService::new();
    service.freeze(index_token)?;
    emit!(AssetFrozen {
        asset_id: index_token.id,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 