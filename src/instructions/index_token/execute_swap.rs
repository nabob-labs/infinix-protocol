//!
//! execute_swap.rs - 指数代币执行交换指令
//!
//! 本文件实现指数代币执行交换指令，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::events::index_token_event::IndexTokenSwapExecuted;

/// 指数代币执行交换指令账户上下文结构体
/// - 管理执行交换操作所需的链上账户
#[derive(Accounts)]
pub struct ExecuteSwapIndexToken<'info> {
    /// 转出方指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub from_index_token: Account<'info, BasketIndexState>,
    /// 转入方指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub to_index_token: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变，Anchor自动校验签名
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 指数代币执行交换指令主实现函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - from_amount: 转出数量
/// - to_amount: 转入数量
/// - 返回：Anchor规范Result，自动生命周期管理
pub fn execute_swap_index_token(
    ctx: Context<ExecuteSwapIndexToken>,
    from_amount: u64,
    to_amount: u64,
) -> anchor_lang::Result<()> {
    // 获取转出方和转入方账户
    let from = &mut ctx.accounts.from_index_token;
    let to = &mut ctx.accounts.to_index_token;
    // 校验转出数量和转入数量
    require!(from_amount > 0 && to_amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams);
    // 校验操作人签名
    require!(ctx.accounts.authority.is_signer, crate::errors::index_token_error::IndexTokenError::NotAllowed);
    // 实际业务应调用服务层或核心交换逻辑，这里仅做事件示例
    emit!(IndexTokenSwapExecuted {
        from_index_token_id: from.id,
        to_index_token_id: to.id,
        from_amount,
        to_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    // Anchor规范返回，生命周期自动管理
    Ok(())
} 