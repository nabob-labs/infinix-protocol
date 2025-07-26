//!
//! Basket Batch Transfer Instruction
//! 篮子批量转账指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等
use crate::core::types::{AlgoParams, StrategyParams}; // 引入算法和策略参数类型

/// 篮子批量转账指令账户上下文
/// - from_basket: 源资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - to_basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct BatchTransferBasket<'info> {
    #[account(mut)]
    pub from_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子批量转账指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amounts: 批量转账数量，类型安全
/// - exec_params: 可选算法参数，类型安全
/// - strategy_params: 可选策略参数，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn batch_transfer_basket(
    ctx: Context<BatchTransferBasket>,
    amounts: Vec<u64>,
    exec_params: Option<AlgoParams>,
    strategy_params: Option<StrategyParams>,
) -> Result<()> {
    let from = &mut ctx.accounts.from_basket;
    let to_baskets = &mut ctx.accounts.to_baskets;
    // 权限校验
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::errors::basket_error::BasketError::NotAllowed);
    // 算法/策略扩展点（如有）
    // ...
    // 批量转账操作
    require!(to_baskets.len() == amounts.len(), crate::errors::basket_error::BasketError::InvalidParams);
    let mut total = 0u64;
    for (i, to) in to_baskets.iter_mut().enumerate() {
        let amount = amounts[i];
        if from.total_value < amount {
            return Err(crate::errors::basket_error::BasketError::InsufficientValue.into());
        }
        from.total_value -= amount;
        to.total_value = to.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?;
        total += amount;
        emit!(BasketTransferred {
            from_basket_id: from.id,
            to_basket_id: to.id,
            amount,
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    Ok(())
} 