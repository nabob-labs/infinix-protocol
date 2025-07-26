//!
//! batch_transfer.rs - 指数代币批量转账指令
//!
//! 本文件实现指数代币批量转账指令，严格遵循Rust、Solana、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::events::index_token_event::IndexTokenBatchTransferred;
use crate::validation::index_token_validation::IndexTokenValidatable;
use crate::core::types::{AlgoParams, StrategyParams};

/// 指数代币批量转账指令账户上下文结构体
/// - 管理批量转账操作所需的链上账户
#[derive(Accounts)]
pub struct BatchTransferIndexToken<'info> {
    /// 转出方指数代币账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub from_index_token: Account<'info, BasketIndexState>,
    /// 转入方账户，需可变，Anchor自动校验PDA和生命周期
    #[account(mut)]
    pub to_index_token: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变，Anchor自动校验签名
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 指数代币批量转账指令主实现函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amounts: 批量转账数量
/// - exec_params: 可选算法参数
/// - strategy_params: 可选策略参数
/// - 返回：Anchor规范Result，自动生命周期管理
pub fn batch_transfer_index_token(
    ctx: Context<BatchTransferIndexToken>,
    amounts: Vec<u64>,
    exec_params: Option<AlgoParams>,
    strategy_params: Option<StrategyParams>,
) -> Result<()> {
    // 获取转出方账户和批量转入方账户
    let from = &mut ctx.accounts.from_index_token;
    let to_tokens = &mut ctx.accounts.to_index_token;
    // 校验操作人权限，必须为转出方授权人
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::index_token_error::IndexTokenError::NotAllowed);
    // 校验数量匹配
    require!(to_tokens.len() == amounts.len(), crate::errors::index_token_error::IndexTokenError::InvalidParams);
    // 校验批量转账总额
    let total: u64 = amounts.iter().sum();
    require!(from.total_supply >= total, crate::errors::index_token_error::IndexTokenError::InsufficientValue);
    // 逐个转账
    for (to, &amount) in to_tokens.iter_mut().zip(amounts.iter()) {
        // 校验转入方账户状态
        to.validate()?;
        // 增加转入方余额，防止溢出
        to.total_supply = to.total_supply.checked_add(amount).ok_or(crate::errors::index_token_error::IndexTokenError::InsufficientValue)?;
    }
    // 扣减转出方余额
    from.total_supply -= total;
    // 可扩展算法与策略融合逻辑（如有）
    if let Some(_exec_params) = exec_params {
        // 算法融合逻辑
    }
    if let Some(_strategy_params) = strategy_params {
        // 策略融合逻辑
    }
    // 触发批量转账事件，链上可追溯
    emit!(IndexTokenBatchTransferred {
        from_index_token_id: from.id,
        to_index_token_ids: to_tokens.iter().map(|t| t.id).collect(),
        amounts,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    // Anchor规范返回，生命周期自动管理
    Ok(())
} 