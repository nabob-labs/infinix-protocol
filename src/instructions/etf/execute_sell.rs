//!
//! ETF Execute Sell Instruction
//! ETF资产执行卖出指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::*;
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产执行卖出指令账户上下文
#[derive(Accounts)]
pub struct ExecuteSellEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub seller: Signer<'info>,
}

/// ETF资产执行卖出指令实现
pub fn execute_sell_etf(ctx: Context<ExecuteSellEtf>, params: TradeParams, price: u64) -> anchor_lang::Result<()> {
    let etf = &mut ctx.accounts.etf;
    etf.validate()?;
    require!(etf.asset_type == crate::core::types::AssetType::ETF, ProgramError::InvalidAssetType);
    // 业务逻辑：调用服务层执行卖出
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.execute_sell(etf, &params, price, ctx.accounts.seller.key())?;
    emit!(EtfSellExecuted {
        etf_id: etf.id,
        amount: params.amount_in,
        price,
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 