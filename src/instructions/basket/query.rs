//!
//! Basket Query Instruction
//! 篮子查询指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::state::baskets::BasketIndexState; // 篮子状态类型
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子查询指令账户上下文
/// - basket: 只读资产篮子账户，类型安全
#[derive(Accounts)]
pub struct QueryBasket<'info> {
    pub basket: Account<'info, BasketIndexState>,
}

/// 篮子查询指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - 返回: Anchor规范Result类型，返回u64类型的篮子总价值
pub fn query_basket(ctx: Context<QueryBasket>) -> anchor_lang::Result<u64> {
    let basket = &ctx.accounts.basket; // 获取只读资产篮子账户
    Ok(basket.total_value) // 返回篮子总价值，Anchor自动处理生命周期
} 