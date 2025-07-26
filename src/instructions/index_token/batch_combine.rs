//!
//! batch_combine.rs - 指数代币批量合并指令
//!
//! 本文件实现指数代币批量合并指令，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::events::index_token_event::IndexTokenBatchCombined;
use crate::validation::index_token_validation::IndexTokenValidatable;
use crate::core::types::BatchTradeParams;

/// 指数代币批量合并指令账户上下文结构体
/// - 管理批量合并操作所需的链上账户
#[derive(Accounts)]
pub struct BatchCombineIndexToken<'info> {
    /// 目标指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub target_index_token: Account<'info, BasketIndexState>,
    /// 源指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub source_index_token: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变，Anchor自动校验签名
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 指数代币批量合并指令主实现函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 批量交易参数
/// - 返回：Anchor规范Result，自动生命周期管理
pub fn batch_combine_index_token(
    ctx: Context<BatchCombineIndexToken>,
    params: BatchTradeParams,
) -> Result<()> {
    // 获取目标和批量源指数代币账户
    let target = &mut ctx.accounts.target_index_token;
    let source = &mut ctx.accounts.source_index_token;
    // 校验操作人权限，必须为目标账户授权人
    require!(ctx.accounts.authority.key() == target.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed);
    // 校验数量匹配
    require!(params.amounts.len() == 1, crate::errors::index_token_error::IndexTokenError::InvalidParams);
    let amount = params.amounts[0];
    // 校验源账户状态
    source.validate()?;
    // 校验源账户余额充足
    require!(source.total_supply >= amount, crate::errors::index_token_error::IndexTokenError::InsufficientValue);
    // 扣减源账户余额
    source.total_supply -= amount;
    // 增加目标账户余额，防止溢出
    target.total_supply = target.total_supply.checked_add(amount).ok_or(crate::errors::index_token_error::IndexTokenError::InsufficientValue)?;
    // 触发批量合并事件，链上可追溯
    emit!(IndexTokenBatchCombined {
        target_index_token_id: target.id,
        source_index_token_ids: vec![source.id],
        amounts: vec![amount],
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    // Anchor规范返回，生命周期自动管理
    Ok(())
} 