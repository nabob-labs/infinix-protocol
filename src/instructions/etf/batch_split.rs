//!
//! ETF Batch Split Instruction
//! ETF资产批量拆分指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::BatchTradeParams;
use crate::services::portfolio_service::PortfolioServiceFacade;

#[event]
pub struct EtfBatchSplit {
    pub source_etf_id: Pubkey,
    pub new_etf_ids: Vec<Pubkey>,
    pub amounts: Vec<u64>,
    pub authority: Pubkey,
    pub timestamp: i64,
}

/// ETF资产批量拆分指令账户上下文
#[derive(Accounts)]
pub struct BatchSplitEtf<'info> {
    #[account(mut)]
    pub source: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产批量拆分指令实现
pub fn batch_split_etf(ctx: Context<BatchSplitEtf>, amounts: Vec<u64>, params: BatchTradeParams) -> anchor_lang::Result<()> {
    let source = &mut ctx.accounts.source;
    source.validate()?;
    require!(source.asset_type == crate::core::types::AssetType::ETF, ProgramError::InvalidAssetType);
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), ProgramError::InvalidAuthority);
    
    // TODO: Handle new_etfs accounts separately - they need to be passed as remaining_accounts
    // For now, just validate the source account
    let facade = PortfolioServiceFacade::new();
    // facade.asset_manage.batch_split(source, &mut new_refs, &amounts, ctx.accounts.authority.key())?;
    
    emit!(EtfBatchSplit {
        source_etf_id: source.id,
        new_etf_ids: vec![], // TODO: Get from remaining_accounts
        amounts: amounts.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 