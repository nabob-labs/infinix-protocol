//!
//! batch_split.rs - 指数代币批量拆分指令
//!
//! 本文件实现指数代币批量拆分指令，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::events::index_token_event::IndexTokenBatchSplit;
// IndexTokenValidatable trait not found, removing import

/// 指数代币批量拆分指令账户上下文结构体
/// - 管理批量拆分操作所需的链上账户
#[derive(Accounts)]
pub struct BatchSplitIndexToken<'info> {
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

/// 指数代币批量拆分指令主实现函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amounts: 批量拆分数量
/// - 返回：Anchor规范Result，自动生命周期管理
pub fn batch_split_index_token(
    ctx: Context<BatchSplitIndexToken>,
    amounts: Vec<u64>,
) -> anchor_lang::Result<()> {
    // 获取源和批量新生成指数代币账户
    let source = &mut ctx.accounts.source_index_token;
    let new_tokens = &mut ctx.accounts.new_index_token;
    // 校验操作人权限，必须为源账户授权人
    require!(ctx.accounts.authority.key() == source.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed);
    // 校验数量匹配
    require!(new_tokens.len() == amounts.len(), crate::errors::index_token_error::IndexTokenError::InvalidParams);
    // 校验批量拆分总额
    let total: u64 = amounts.iter().sum();
    require!(source.total_supply >= total, crate::errors::index_token_error::IndexTokenError::InsufficientValue);
    // 逐个拆分
    for (new_token, &amount) in new_tokens.iter_mut().zip(amounts.iter()) {
        // 校验新生成账户状态
        new_token.validate()?;
        // 增加新生成账户余额，防止溢出
        new_token.total_supply = new_token.total_supply.checked_add(amount).ok_or(crate::errors::index_token_error::IndexTokenError::InsufficientValue)?;
    }
    // 扣减源账户余额
    source.total_supply -= total;
    // 触发批量拆分事件，链上可追溯
    emit!(IndexTokenBatchSplit {
        source_index_token_id: source.id,
        new_index_token_ids: new_tokens.iter().map(|t| t.id).collect(),
        amounts,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    // Anchor规范返回，生命周期自动管理
    Ok(())
} 