//!
//! ETF Adapter Trade Instruction
//! ETF资产适配器交易指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::*;
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产适配器交易指令账户上下文
#[derive(Accounts)]
pub struct AdapterTradeEtf<'info> {
    #[account(mut)]
    pub etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产适配器交易指令实现
pub fn adapter_trade_etf(ctx: Context<AdapterTradeEtf>, adapter_params: AdapterParams, trade_params: TradeParams) -> anchor_lang::Result<()> {
    let etf = &mut ctx.accounts.etf;
    etf.validate()?;
    require!(etf.asset_type == crate::core::types::AssetType::ETF, ProgramError::InvalidAssetType);
    require_keys_eq!(etf.authority, ctx.accounts.authority.key(), ProgramError::InvalidAuthority);
    // 业务逻辑：适配器交易
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.adapter_trade(etf, &adapter_params, &trade_params, ctx.accounts.authority.key())?;
    emit!(EtfAdapterTraded {
        etf_id: etf.id,
        adapter_name: adapter_params.adapter_name.to_string(),
        params: adapter_params.params.to_vec(),
        trade_params: trade_params,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 