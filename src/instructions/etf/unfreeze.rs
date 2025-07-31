//! ETF资产unfreeze指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetUnfrozen;

/// ETF资产unfreeze指令账户上下文
#[derive(Accounts)]
pub struct UnfreezeEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>, // ETF资产账户，需可变
    pub authority: Signer<'info>,             // 操作人签名者
}

/// ETF资产unfreeze指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
pub fn unfreeze_etf(ctx: Context<UnfreezeEtf>) -> anchor_lang::Result<()> {
    let etf = &mut ctx.accounts.etf;
    require!(etf.asset_type == AssetType::ETF, ProgramError::InvalidAssetType);
    let service = EtfService::new();
    service.unfreeze(etf)?;
    emit!(AssetUnfrozen {
        asset_id: etf.id,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 