//! IndexToken资产mint指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetMinted;

/// IndexToken资产mint指令账户上下文
#[derive(Accounts)]
pub struct MintIndexToken<'info> {
    #[account(mut)]
    pub index_token: Account<'info, BasketIndexState>, // IndexToken资产账户，需可变
    pub authority: Signer<'info>,                     // 操作人签名者
}

/// IndexToken资产mint指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 增发数量，类型安全
pub fn mint_index_token(ctx: Context<MintIndexToken>, amount: u64) -> Result<()> {
    let index_token = &mut ctx.accounts.index_token;
    require!(index_token.asset_type == AssetType::IndexToken, crate::error::ProgramError::InvalidAssetType);
    let service = IndexTokenService::new();
    service.mint(index_token, amount)?;
    emit!(AssetMinted {
        basket_id: index_token.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 