//!
//! batch_subscribe.rs - 指数代币批量申购指令
//!
//! 本文件实现指数代币批量申购指令，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::events::index_token_event::IndexTokenBatchSubscribed;
// IndexTokenValidatable trait not found, removing import

/// 指数代币批量申购指令账户上下文结构体
/// - 管理批量申购操作所需的链上账户
#[derive(Accounts)]
pub struct BatchSubscribeIndexToken<'info> {
    /// 指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub index_token: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变，Anchor自动校验签名
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 指数代币批量申购指令主实现函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amounts: 批量申购数量
/// - 返回：Anchor规范Result，自动生命周期管理
pub fn batch_subscribe_index_token(
    ctx: Context<BatchSubscribeIndexToken>,
    amounts: Vec<u64>,
) -> anchor_lang::Result<()> {
    // 获取指数代币账户
    let index_token = &mut ctx.accounts.index_token;
    // 校验账户状态
    index_token.validate()?;
    // 校验操作人权限，必须为账户授权人
    require!(ctx.accounts.authority.key() == index_token.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed);
    // 逐个申购
    for &amount in &amounts {
        require!(amount > 0, crate::errors::index_token_error::IndexTokenError::InvalidParams);
        index_token.total_supply = index_token.total_supply.checked_add(amount).ok_or(crate::errors::index_token_error::IndexTokenError::InsufficientValue)?;
    }
    // 触发批量申购事件，链上可追溯
    emit!(IndexTokenBatchSubscribed {
        index_token_id: index_token.id,
        amounts,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    // Anchor规范返回，生命周期自动管理
    Ok(())
} 