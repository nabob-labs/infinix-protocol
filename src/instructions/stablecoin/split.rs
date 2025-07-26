//! Stablecoin资产split指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetSplit;

/// Stablecoin资产split指令账户上下文
#[derive(Accounts)]
pub struct SplitStablecoin<'info> {
    #[account(mut)]
    pub source_stablecoin: Account<'info, BasketIndexState>, // 源Stablecoin资产账户，需可变
    #[account(mut)]
    pub new_stablecoin: Account<'info, BasketIndexState>,    // 新Stablecoin资产账户，需可变
    pub authority: Signer<'info>,                           // 操作人签名者
}

/// Stablecoin资产split指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 拆分数量，类型安全
pub fn split_stablecoin(ctx: Context<SplitStablecoin>, amount: u64) -> Result<()> {
    let source = &mut ctx.accounts.source_stablecoin;
    let new = &mut ctx.accounts.new_stablecoin;
    require!(source.asset_type == AssetType::Stablecoin && new.asset_type == AssetType::Stablecoin, crate::error::ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    service.split(source, new, amount)?;
    emit!(AssetSplit {
        source_asset_id: source.id,
        new_asset_id: new.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 