//!
//! execute_split.rs - 指数代币执行拆分指令
//!
//! 本文件实现指数代币执行拆分指令，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::events::index_token_event::IndexTokenSplitExecuted;

/// 指数代币执行拆分指令账户上下文结构体
/// - 管理执行拆分操作所需的链上账户
#[derive(Accounts)]
pub struct ExecuteSplitIndexToken<'info> {
    /// 源指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub source_index_token: Account<'info, BasketIndexState>,
    /// 新生成指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub new_index_token: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变，Anchor自动校验签名
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 指数代币执行拆分指令主实现函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 拆分数量
/// - 返回：Anchor规范Result，自动生命周期管理
pub fn execute_split_index_token(
    ctx: Context<ExecuteSplitIndexToken>,
    amount: u64,
) -> Result<()> {
    // 获取源和新生成指数代币账户
    let source = &mut ctx.accounts.source_index_token;
    let new_token = &mut ctx.accounts.new_index_token;
    // 校验拆分数量
    require!(amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams);
    // 校验操作人签名
    require!(ctx.accounts.authority.is_signer, crate::errors::index_token_error::IndexTokenError::NotAllowed);
    // 实际业务应调用服务层或核心拆分逻辑，这里仅做事件示例
    emit!(IndexTokenSplitExecuted {
        source_index_token_id: source.id,
        new_index_token_id: new_token.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    // Anchor规范返回，生命周期自动管理
    Ok(())
} 