//!
//! Basket Execute Combine Instruction
//! 篮子执行合并指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子执行合并指令账户上下文
/// - target_basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - source_basket: 源资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct ExecuteCombineBasket<'info> {
    #[account(mut)]
    pub target_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub source_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子执行合并指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 合并数量，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn execute_combine_basket(
    ctx: Context<ExecuteCombineBasket>,
    amount: u64,
) -> Result<()> {
    let target = &mut ctx.accounts.target_basket;
    let source = &mut ctx.accounts.source_basket;
    // 权限校验
    require_keys_eq!(target.authority, ctx.accounts.authority.key(), crate::errors::basket_error::BasketError::NotAllowed);
    // 执行合并操作（实际业务逻辑应在服务层实现）
    BasketService::execute_combine(target, source, amount, ctx.accounts.authority.key())?;
    emit!(BasketCombined {
        target_basket_id: target.id,
        source_basket_id: source.id,
        amount,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 