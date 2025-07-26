//!
//! ETF Batch Split Instruction
//! ETF资产批量拆分指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::BatchTradeParams;
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产批量拆分指令账户上下文
#[derive(Accounts)]
pub struct BatchSplitEtf<'info> {
    #[account(mut)]
    pub source: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub new_etfs: Vec<Account<'info, BasketIndexState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产批量拆分指令实现
pub fn batch_split_etf(ctx: Context<BatchSplitEtf>, amounts: Vec<u64>, params: BatchTradeParams) -> Result<()> {
    let source = &mut ctx.accounts.source;
    let new_etfs = &mut ctx.accounts.new_etfs;
    source.validate()?;
    for new_etf in new_etfs.iter_mut() {
        new_etf.validate()?;
    }
    require!(source.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    for new_etf in new_etfs.iter() {
        require!(new_etf.asset_type == crate::core::types::AssetType::ETF, crate::error::ProgramError::InvalidAssetType);
    }
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：批量拆分
    let mut new_refs: Vec<&mut BasketIndexState> = new_etfs.iter_mut().map(|a| a.as_mut()).collect();
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.batch_split(source, &mut new_refs, &amounts, ctx.accounts.authority.key())?;
    emit!(EtfBatchSplit {
        source_etf_id: source.id,
        new_etf_ids: new_etfs.iter().map(|a| a.id).collect(),
        amounts: amounts.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 