//! ETF资产authorize指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::AssetType;
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetAuthorized;

/// ETF资产authorize指令账户上下文
#[derive(Accounts)]
pub struct AuthorizeEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>, // ETF资产账户，需可变
    pub authority: Signer<'info>,             // 操作人签名者
}

/// ETF资产authorize指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - new_authority: 新授权人公钥
pub fn authorize_etf(ctx: Context<AuthorizeEtf>, new_authority: Pubkey) -> Result<()> {
    let etf = &mut ctx.accounts.etf;
    require!(etf.asset_type == AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    let service = EtfService::new();
    service.authorize(etf, new_authority)?;
    emit!(AssetAuthorized {
        asset_id: etf.id,
        new_authority,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 