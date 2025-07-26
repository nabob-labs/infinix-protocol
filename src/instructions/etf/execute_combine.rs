//!
//! ETF Execute Combine Instruction
//! ETF资产执行合并指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::TradeParams;
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产执行合并指令账户上下文
#[derive(Accounts)]
pub struct ExecuteCombineEtf<'info> {
    #[account(mut)]
    pub target: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub source: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产执行合并指令实现
pub fn execute_combine_etf(ctx: Context<ExecuteCombineEtf>, amount: u64, params: TradeParams) -> Result<()> {
    let target = &mut ctx.accounts.target;
    let source = &mut ctx.accounts.source;
    target.validate()?;
    source.validate()?;
    require!(target.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require!(source.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：调用服务层执行合并
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.execute_combine(target, source, amount, &params, ctx.accounts.authority.key())?;
    emit!(EtfCombineExecuted {
        target_etf_id: target.id,
        source_etf_id: source.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 