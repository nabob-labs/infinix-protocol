//!
//! ETF Execute Split Instruction
//! ETF资产执行拆分指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::TradeParams;
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产执行拆分指令账户上下文
#[derive(Accounts)]
pub struct ExecuteSplitEtf<'info> {
    #[account(mut)]
    pub source: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub new_etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产执行拆分指令实现
pub fn execute_split_etf(ctx: Context<ExecuteSplitEtf>, amount: u64, params: TradeParams) -> Result<()> {
    let source = &mut ctx.accounts.source;
    let new_etf = &mut ctx.accounts.new_etf;
    source.validate()?;
    new_etf.validate()?;
    require!(source.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require!(new_etf.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：调用服务层执行拆分
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.execute_split(source, new_etf, amount, &params, ctx.accounts.authority.key())?;
    emit!(EtfSplitExecuted {
        source_etf_id: source.id,
        new_etf_id: new_etf.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 