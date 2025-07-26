//!
//! ETF Algo Trade Instruction
//! ETF资产算法交易指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{AlgoParams, TradeParams};
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产算法交易指令账户上下文
#[derive(Accounts)]
pub struct AlgoTradeEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产算法交易指令实现
pub fn algo_trade_etf(ctx: Context<AlgoTradeEtf>, algo_params: AlgoParams, trade_params: TradeParams) -> Result<()> {
    let etf = &mut ctx.accounts.etf;
    etf.validate()?;
    require!(etf.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(etf.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：算法交易
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.algo_trade(etf, &algo_params, &trade_params, ctx.accounts.authority.key())?;
    emit!(EtfAlgoTraded {
        etf_id: etf.id,
        algo_name: algo_params.algo_name.to_string(),
        params: algo_params.params.to_vec(),
        trade_params: trade_params,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 