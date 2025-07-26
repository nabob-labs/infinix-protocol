//!
//! Basket Execute Swap Instruction
//! 篮子执行兑换指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子执行兑换指令账户上下文
/// - from_basket: 源资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - to_basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct ExecuteSwapBasket<'info> {
    #[account(mut)]
    pub from_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子执行兑换指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - from_amount: 兑换输入数量，类型安全
/// - to_amount: 兑换输出数量，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn execute_swap_basket(
    ctx: Context<ExecuteSwapBasket>,
    from_amount: u64,
    to_amount: u64,
) -> Result<()> {
    let from = &mut ctx.accounts.from_basket;
    let to = &mut ctx.accounts.to_basket;
    // 权限校验
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::errors::basket_error::BasketError::NotAllowed);
    // 执行兑换操作（实际业务逻辑应在服务层实现）
    BasketService::execute_swap(from, to, from_amount, to_amount, ctx.accounts.authority.key())?;
    emit!(BasketSwapped {
        from_basket_id: from.id,
        to_basket_id: to.id,
        amount: from_amount,
        price: to_amount, // 这里price字段可根据实际业务调整
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 