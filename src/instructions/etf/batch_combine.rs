//!
//! ETF Batch Combine Instruction
//! ETF资产批量合并指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::BatchTradeParams;
use crate::services::portfolio_service::PortfolioServiceFacade;

/// ETF资产批量合并指令账户上下文
#[derive(Accounts)]
pub struct BatchCombineEtf<'info> {
    #[account(mut)]
    pub target: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub source_etf: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// ETF资产批量合并指令实现
pub fn batch_combine_etf(ctx: Context<BatchCombineEtf>, params: BatchTradeParams) -> anchor_lang::Result<()> {
    let target = &mut ctx.accounts.target;
    let source_etf = &mut ctx.accounts.source_etf;
    target.validate()?;
    source_etf.validate()?;
    require!(target.asset_type == crate::core::types::AssetType::ETF, ProgramError::InvalidAssetType);
    require!(source_etf.asset_type == crate::core::types::AssetType::ETF, ProgramError::InvalidAssetType);
    require_keys_eq!(ctx.accounts.authority.key(), target.authority, ProgramError::InvalidAuthority);
    // 业务逻辑：批量合并
    let mut source_refs: Vec<&mut BasketIndexState> = vec![source_etf.as_mut()];
    let facade = PortfolioServiceFacade::new();
    facade.asset_manage.batch_combine(target, &mut source_refs, &params, ctx.accounts.authority.key())?;
    emit!(EtfBatchCombined {
        target_etf_id: target.id,
        source_etf_ids: vec![source_etf.id],
        amounts: params.trades.iter().map(|s| s.from_amount).collect(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 