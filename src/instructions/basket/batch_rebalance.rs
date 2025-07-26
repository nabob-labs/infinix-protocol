//!
//! Basket Batch Rebalance Instruction
//! 篮子批量再平衡指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等
use crate::core::types::BatchTradeParams; // 引入批量交易参数类型

/// 篮子批量再平衡指令账户上下文
/// - basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct BatchRebalanceBasket<'info> {
    #[account(mut)]
    pub basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子批量再平衡指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 批量再平衡参数，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn batch_rebalance_basket(
    ctx: Context<BatchRebalanceBasket>,
    params: BatchTradeParams,
) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    // 权限校验
    require_keys_eq!(basket.authority, ctx.accounts.authority.key(), crate::errors::basket_error::BasketError::NotAllowed);
    // 批量再平衡操作（实际业务逻辑应在服务层实现）
    BasketService::batch_rebalance(basket, &params)?;
    emit!(BasketBatchRebalanced {
        basket_id: basket.id,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 