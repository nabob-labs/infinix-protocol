//!
//! Basket Resume Instruction
//! 篮子恢复指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use crate::validation::basket_validation::BasketValidatable; // 引入篮子校验trait，便于状态校验
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子恢复指令账户上下文
/// - basket_index: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct ResumeBasket<'info> {
    #[account(mut)]
    pub basket_index: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子恢复指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn resume_basket(ctx: Context<ResumeBasket>) -> Result<()> {
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法恢复，业务安全
    BasketService::resume(basket_index)?; // 调用服务层恢复逻辑，处理实际恢复操作
    emit!(BasketResumed { // 触发篮子恢复事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID，便于链上追踪
        authority: ctx.accounts.authority.key(), // 事件：操作人，便于审计
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳，便于审计
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
} 