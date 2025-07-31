//! RWA资产freeze指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetFrozen;

/// RWA资产freeze指令账户上下文
#[derive(Accounts)]
pub struct FreezeRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>, // RWA资产账户，需可变
    pub authority: Signer<'info>,             // 操作人签名者
}

/// RWA资产freeze指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
pub fn freeze_rwa(ctx: Context<FreezeRwa>) -> anchor_lang::Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    require!(rwa.asset_type == AssetType::RWA, ProgramError::InvalidAssetType);
    let service = RwaService::new();
    service.freeze(rwa)?;
    emit!(AssetFrozen {
        asset_id: rwa.id,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 