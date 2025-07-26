//!
//! ETF Strategy Trade Instruction
//! ETF资产策略交易指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{StrategyParams, TradeParams, OracleParams};
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产策略交易参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StrategyTradeEtfParams {
    /// 策略参数
    pub strategy: StrategyParams,
    /// 交换参数（可选）
    pub swap_params: Option<TradeParams>,
    /// 价格参数（可选）
    pub price_params: Option<OracleParams>,
    /// 执行参数（可选）
    pub exec_params: Option<TradeParams>,
}

/// ETF资产策略交易指令账户上下文
#[derive(Accounts)]
pub struct StrategyTradeEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产策略交易指令实现
pub fn strategy_trade_etf(ctx: Context<StrategyTradeEtf>, params: StrategyTradeEtfParams) -> Result<()> {
    let etf = &mut ctx.accounts.etf;
    etf.validate()?;
    require!(etf.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(etf.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：策略交易
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.strategy_trade(etf, &params, ctx.accounts.authority.key())?;
    emit!(EtfStrategyTraded {
        etf_id: etf.id,
        strategy: params.strategy.strategy_name.to_string(),
        params: params.strategy.params.to_vec(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 