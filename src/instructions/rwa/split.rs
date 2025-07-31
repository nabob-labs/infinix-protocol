//! RWA资产split指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetSplit;

/// RWA资产split指令账户上下文
#[derive(Accounts)]
pub struct SplitRwa<'info> {
    #[account(mut)]
    pub source_rwa: Account<'info, BasketIndexState>, // 源RWA资产账户，需可变
    #[account(mut)]
    pub new_rwa: Account<'info, BasketIndexState>,    // 新RWA资产账户，需可变
    pub authority: Signer<'info>,                    // 操作人签名者
}

/// RWA资产split指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 拆分数量，类型安全
pub fn split_rwa(ctx: Context<SplitRwa>, amount: u64) -> anchor_lang::Result<()> {
    let source = &mut ctx.accounts.source_rwa;
    let new = &mut ctx.accounts.new_rwa;
    require!(source.asset_type == AssetType::RWA && new.asset_type == AssetType::RWA, ProgramError::InvalidAssetType);
    let service = RwaService::new();
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