//!
//! RWA Execute Buy Instruction
//! RWA资产执行买入指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use anchor_lang::prelude::*;
use crate::core::types::*;

/// RWA资产执行买入指令账户上下文
#[derive(Accounts)]
pub struct ExecuteBuyRwa<'info> {
    #[account(mut)]
    pub rwa: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub buyer: Signer<'info>,
}

/// RWA资产执行买入指令实现
pub fn execute_buy_rwa(ctx: Context<ExecuteBuyRwa>, params: TradeParams, price: u64) -> anchor_lang::Result<()> {
    let rwa = &mut ctx.accounts.rwa;
    rwa.validate()?;
    require!(rwa.asset_type == AssetType::RWA, ProgramError::InvalidAssetType);
    // 业务逻辑：调用服务层执行买入
    // TODO: 调用RwaService::execute_buy(rwa, &params, price, ctx.accounts.buyer.key())
    emit!(RwaBuyExecuted {
        rwa_id: rwa.id,
        amount: params.amount_in,
        price,
        buyer: ctx.accounts.buyer.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 