//! ETF资产sell指令
//! Anchor最小功能单元，生产级注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetSold;

/// ETF资产sell指令账户上下文
#[derive(Accounts)]
pub struct SellEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>, // ETF资产账户，需可变
    pub authority: Signer<'info>,             // 操作人签名者
}

/// ETF资产sell指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 卖出数量，类型安全
pub fn sell_etf(ctx: Context<SellEtf>, amount: u64) -> anchor_lang::Result<()> {
    let etf = &mut ctx.accounts.etf;
    require!(etf.asset_type == AssetType::ETF, ProgramError::InvalidAssetType);
    let service = EtfService::new();
    service.sell(etf, amount)?;
    emit!(AssetSold {
        basket_id: etf.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 