//!
//! ETF Batch Swap Instruction
//! ETF资产批量swap指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::BatchTradeParams;
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产批量swap指令账户上下文
#[derive(Accounts)]
pub struct BatchSwapEtf<'info> {
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to_etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产批量swap指令实现
pub fn batch_swap_etf(ctx: Context<BatchSwapEtf>, params: BatchTradeParams) -> Result<()> {
    let from = &mut ctx.accounts.from;
    let to_etf = &mut ctx.accounts.to_etf;
    from.validate()?;
    to_etf.validate()?;
    require!(from.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require!(to_etf.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：批量swap
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.batch_swap(from, to_etf, &params, ctx.accounts.authority.key())?;
    emit!(EtfBatchSwapped {
        from_etf_id: from.id,
        to_etf_id: to_etf.id,
        from_amounts: params.trades.iter().map(|s| s.from_amount).collect(),
        to_amounts: params.trades.iter().map(|s| s.to_amount).collect(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 