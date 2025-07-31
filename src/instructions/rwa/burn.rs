//! RWA资产burn指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetBurned;

/// RWA资产burn指令账户上下文
#[derive(Accounts)]
pub struct BurnRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>, // RWA资产账户，需可变
    pub authority: Signer<'info>, // 操作人签名者
}

/// RWA资产burn指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 销毁数量，类型安全
pub fn burn_rwa(ctx: Context<BurnRwa>, amount: u64) -> anchor_lang::Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    require!(rwa.asset_type == AssetType::RWA, ProgramError::InvalidAssetType);
    let service = RwaService::new();
    service.burn(rwa, amount)?;
    emit!(AssetBurned {
        basket_id: rwa.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 