//!
//! RWA Execute Sell Instruction
//! RWA资产执行卖出指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{TradeParams, OracleParams, AssetType};

/// RWA资产执行卖出指令账户上下文
#[derive(Accounts)]
pub struct ExecuteSellRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub seller: Signer<'info>,
}

/// RWA资产执行卖出指令实现
pub fn execute_sell_rwa(ctx: Context<ExecuteSellRwa>, params: TradeParams, price: u64) -> Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    rwa.validate()?;
    require!(rwa.asset_type == AssetType::RWA, crate::error::ProgramError::InvalidAssetType);
    // 业务逻辑：调用服务层执行卖出
    // TODO: 调用RwaService::execute_sell(rwa, &params, price, ctx.accounts.seller.key())
    emit!(RwaSellExecuted {
        rwa_id: rwa.id,
        amount: params.amount_in,
        price,
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 