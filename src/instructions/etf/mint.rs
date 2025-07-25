//! ETF资产mint指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetMinted;

/// ETF资产mint指令账户上下文
#[derive(Accounts)]
pub struct MintEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>, // ETF资产账户，需可变
    pub authority: Signer<'info>,             // 操作人签名者
}

/// ETF资产mint指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 增发数量，类型安全
pub fn mint_etf(ctx: Context<MintEtf>, amount: u64) -> Result<()> {
    let etf = &mut ctx.accounts.etf;
    require!(etf.asset_type == AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    let service = EtfService::new();
    service.mint(etf, amount)?;
    emit!(AssetMinted {
        basket_id: etf.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 