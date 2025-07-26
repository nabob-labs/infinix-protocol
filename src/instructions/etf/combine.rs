//! ETF资产combine指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetCombined;

/// ETF资产combine指令账户上下文
#[derive(Accounts)]
pub struct CombineEtf<'info> {
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>, // 被合并账户，需可变
    #[account(mut)]
    pub to: Account<'info, BasketIndexState>,   // 目标账户，需可变
    pub authority: Signer<'info>,              // 操作人签名者
}

/// ETF资产combine指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 合并数量，类型安全
pub fn combine_etf(ctx: Context<CombineEtf>, amount: u64) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to = &mut ctx.accounts.to;
    require!(from.asset_type == AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require!(to.asset_type == AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    let service = EtfService::new();
    service.combine(from, to, amount)?;
    emit!(AssetCombined {
        from_id: from.id,
        to_id: to.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 