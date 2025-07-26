//!
//! ETF Batch Transfer Instruction
//! ETF资产批量转账指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::TradeParams;
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产批量转账指令账户上下文
#[derive(Accounts)]
pub struct BatchTransferEtf<'info> {
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to_etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产批量转账指令实现
pub fn batch_transfer_etf(ctx: Context<BatchTransferEtf>, amounts: Vec<u64>, params: TradeParams) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to_etf = &mut ctx.accounts.to_etf;
    from.validate()?;
    to_etf.validate()?;
    require!(from.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require!(to_etf.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：批量转账
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.batch_transfer(from, to_etf, &amounts, ctx.accounts.authority.key())?;
    emit!(EtfBatchTransferred {
        from_etf_id: from.id,
        to_etf_id: to_etf.id,
        amounts: amounts.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 