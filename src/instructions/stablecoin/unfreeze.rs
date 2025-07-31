//! Stablecoin资产unfreeze指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetUnfrozen;

/// Stablecoin资产unfreeze指令账户上下文
#[derive(Accounts)]
pub struct UnfreezeStablecoin<'info> {
    #[account(mut)]
    pub stablecoin: Account<'info, BasketIndexState>, // Stablecoin资产账户，需可变
    pub authority: Signer<'info>,                    // 操作人签名者
}

/// Stablecoin资产unfreeze指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
pub fn unfreeze_stablecoin(ctx: Context<UnfreezeStablecoin>) -> anchor_lang::Result<()> {
    let stablecoin = &mut ctx.accounts.stablecoin;
    require!(stablecoin.asset_type == AssetType::Stablecoin, ProgramError::InvalidAssetType);
    let service = StablecoinService::new();
    service.unfreeze(stablecoin)?;
    emit!(AssetUnfrozen {
        asset_id: stablecoin.id,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 