//!
//! ETF Execute Buy Instruction
//! ETF资产执行买入指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{TradeParams, OracleParams};
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产执行买入指令账户上下文
#[derive(Accounts)]
pub struct ExecuteBuyEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub buyer: Signer<'info>,
}

/// ETF资产执行买入指令实现
pub fn execute_buy_etf(ctx: Context<ExecuteBuyEtf>, params: TradeParams, price: u64) -> Result<()> {
    let etf = &mut ctx.accounts.etf;
    etf.validate()?;
    require!(etf.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    // 业务逻辑：调用服务层执行买入
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.execute_buy(etf, &params, price, ctx.accounts.buyer.key())?;
    emit!(EtfBuyExecuted {
        etf_id: etf.id,
        amount: params.amount_in,
        price,
        buyer: ctx.accounts.buyer.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 