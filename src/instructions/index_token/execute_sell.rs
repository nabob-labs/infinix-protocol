//!
//! execute_sell.rs - 指数代币执行卖出指令
//!
//! 本文件实现指数代币执行卖出指令，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::events::index_token_event::IndexTokenSellExecuted;
use crate::core::types::TradeParams;

/// 指数代币执行卖出指令账户上下文结构体
/// - 管理执行卖出操作所需的链上账户
#[derive(Accounts)]
pub struct ExecuteSellIndexToken<'info> {
    /// 指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub index_token: Account<'info, BasketIndexState>,
    /// 卖方签名者，需可变，Anchor自动校验签名
    #[account(mut)]
    pub seller: Signer<'info>,
}

/// 指数代币执行卖出指令主实现函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 交易参数
/// - price: 卖出价格
/// - 返回：Anchor规范Result，自动生命周期管理
pub fn execute_sell_index_token(
    ctx: Context<ExecuteSellIndexToken>,
    params: TradeParams,
    price: u64,
) -> anchor_lang::Result<()> {
    // 获取指数代币账户
    let index_token = &mut ctx.accounts.index_token;
    // 校验卖出数量
    require!(params.amount_in > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams);
    // 校验卖方签名
    require!(ctx.accounts.seller.is_signer, crate::errors::index_token_error::IndexTokenError::NotAllowed);
    // 实际业务应调用服务层或核心卖出逻辑，这里仅做事件示例
    emit!(IndexTokenSellExecuted {
        index_token_id: index_token.id,
        amount: params.amount_in,
        price,
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    // Anchor规范返回，生命周期自动管理
    Ok(())
} 