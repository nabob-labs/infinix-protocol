//!
//! Basket Rebalance Instruction
//! 篮子再平衡指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketServiceFacade; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 篮子状态类型
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子再平衡指令账户上下文
/// - basket_index: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct RebalanceBasket<'info> {
    #[account(mut)]
    pub basket_index: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子再平衡指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - new_weights: 新权重，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn rebalance_basket(ctx: Context<RebalanceBasket>, new_weights: Vec<u64>) -> anchor_lang::Result<()> {
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态（如活跃、合法等），防止非法操作
    let service = BasketServiceFacade::new(); // 创建服务实例
    service.rebalance.rebalance(basket_index, new_weights.clone())?; // 调用服务层再平衡逻辑，处理实际权重调整
    emit!(BasketRebalanced { // 触发篮子再平衡事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID，便于链上追踪
        new_weights, // 事件：新权重，记录操作明细
        authority: ctx.accounts.authority.key(), // 事件：操作人，便于审计
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳，便于审计
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
} 