//!
//! RWA Execute Split Instruction
//! RWA资产执行拆分指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{TradeParams, AssetType};

/// RWA资产执行拆分指令账户上下文
#[derive(Accounts)]
pub struct ExecuteSplitRwa<'info> {
    #[account(mut)]
    pub source: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub new_rwa: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// RWA资产执行拆分指令实现
pub fn execute_split_rwa(ctx: Context<ExecuteSplitRwa>, amount: u64, params: TradeParams) -> Result<()> {
    let source = &mut ctx.accounts.source;
    let new_rwa = &mut ctx.accounts.new_rwa;
    source.validate()?;
    new_rwa.validate()?;
    require!(source.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    require!(new_rwa.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    require_keys_eq!(source.authority, ctx.accounts.authority.key(), crate::error::ProgramError::InvalidAuthority);
    // 业务逻辑：调用服务层执行拆分
    // TODO: 调用RwaService::execute_split(source, new_rwa, amount, &params, ctx.accounts.authority.key())
    emit!(RwaSplitExecuted {
        source_rwa_id: source.id,
        new_rwa_id: new_rwa.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 