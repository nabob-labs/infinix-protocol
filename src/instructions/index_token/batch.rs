//! IndexToken资产batch指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetBatchProcessed;

/// IndexToken资产batch指令账户上下文
#[derive(Accounts)]
pub struct BatchIndexToken<'info> {
    #[account(mut)]
    pub index_token: Account<'info, BasketIndexState>, // IndexToken资产账户，需可变
    pub authority: Signer<'info>,                     // 操作人签名者
}

/// IndexToken资产batch指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - actions: 批量操作类型与参数数组
pub fn batch_index_token(ctx: Context<BatchIndexToken>, actions: Vec<IndexTokenBatchAction>) -> Result<()> {
    let index_token = &mut ctx.accounts.index_token;
    require!(index_token.asset_type == AssetType::IndexToken, crate::error::ProgramError::InvalidAssetType);
    let service = IndexTokenService::new();
    service.batch(index_token, &actions)?;
    emit!(AssetBatchProcessed {
        asset_id: index_token.id,
        actions_count: actions.len() as u64,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

/// 批量操作类型定义
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IndexTokenBatchAction {
    pub action_type: u8, // 0:mint, 1:burn, 2:transfer, 3:freeze, 4:unfreeze, ...
    pub amount: u64,
    pub target: Option<Pubkey>, // 目标账户（如转账、授权等）
} 