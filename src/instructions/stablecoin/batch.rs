//! Stablecoin资产batch指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetBatchProcessed;

/// Stablecoin资产batch指令账户上下文
#[derive(Accounts)]
pub struct BatchStablecoin<'info> {
    #[account(mut)]
    pub stablecoin: Account<'info, BasketIndexState>, // Stablecoin资产账户，需可变
    pub authority: Signer<'info>,                    // 操作人签名者
}

/// Stablecoin资产batch指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - actions: 批量操作类型与参数数组
pub fn batch_stablecoin(ctx: Context<BatchStablecoin>, actions: Vec<StablecoinBatchAction>) -> Result<()> {
    let stablecoin = &mut ctx.accounts.stablecoin;
    require!(stablecoin.asset_type == AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    service.batch(stablecoin, &actions)?;
    emit!(AssetBatchProcessed {
        asset_id: stablecoin.id,
        actions_count: actions.len() as u64,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

/// 批量操作类型定义
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StablecoinBatchAction {
    pub action_type: u8, // 0:mint, 1:burn, 2:transfer, 3:freeze, 4:unfreeze, ...
    pub amount: u64,
    pub target: Option<Pubkey>, // 目标账户（如转账、授权等）
} 