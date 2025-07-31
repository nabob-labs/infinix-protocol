//! IndexToken资产sell指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetSold;

/// IndexToken资产sell指令账户上下文
#[derive(Accounts)]
pub struct SellIndexToken<'info> {
    #[account(mut)]
    pub index_token: Account<'info, BasketIndexState>, // IndexToken资产账户，需可变
    pub authority: Signer<'info>,                     // 操作人签名者
}

/// IndexToken资产sell指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 卖出数量，类型安全
pub fn sell_index_token(ctx: Context<SellIndexToken>, amount: u64) -> anchor_lang::Result<()> {
    let index_token = &mut ctx.accounts.index_token;
    require!(index_token.asset_type == AssetType::IndexToken, ProgramError::InvalidAssetType);
    let service = IndexTokenService::new();
    service.sell(index_token, amount)?;
    emit!(AssetSold {
        basket_id: index_token.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 