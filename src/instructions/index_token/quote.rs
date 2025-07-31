//! IndexToken资产quote指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetQuoted;

/// IndexToken资产quote指令账户上下文
#[derive(Accounts)]
pub struct QuoteIndexToken<'info> {
    pub index_token: Account<'info, BasketIndexState>, // IndexToken资产账户
}

/// IndexToken资产quote指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 询价数量
pub fn quote_index_token(ctx: Context<QuoteIndexToken>, amount: u64) -> anchor_lang::Result<()> {
    let index_token = &ctx.accounts.index_token;
    require!(index_token.asset_type == AssetType::IndexToken, ProgramError::InvalidAssetType);
    let service = IndexTokenService::new();
    let quote = service.quote(index_token, amount)?;
    emit!(AssetQuoted {
        asset_id: index_token.id,
        amount,
        quote_value: quote.quote_value,
        price_oracle: quote.price_oracle,
        asset_type: index_token.asset_type as u8,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 