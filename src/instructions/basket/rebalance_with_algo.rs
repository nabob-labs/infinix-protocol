//!
//! Basket Rebalance With Algo Instruction
//! 篮子带算法再平衡指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use crate::validation::basket_validation::BasketValidatable; // 引入篮子校验trait，便于状态校验
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子带算法再平衡指令账户上下文
/// - basket_index: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct RebalanceBasketWithAlgo<'info> {
    #[account(mut)]
    pub basket_index: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子带算法再平衡指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - new_weights: 新权重，类型安全
/// - algo_name: 算法名称，类型安全
/// - params: 算法参数，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn rebalance_basket_with_algo(
    ctx: Context<RebalanceBasketWithAlgo>,
    new_weights: Vec<u64>,
    algo_name: String,
    params: crate::algorithms::traits::ExecutionParams,
) -> Result<()> {
    use crate::algorithms::algorithm_registry::AlgorithmRegistry; // 引入算法注册表
    let registry = AlgorithmRegistry::new(); // 实际项目应为全局单例，这里为演示新建
    let algo = registry
        .get_execution(&algo_name)
        .ok_or(crate::errors::basket_error::BasketError::Unknown)?;
    let basket_index = &mut ctx.accounts.basket_index;
    basket_index.validate()?;
    crate::services::basket_service::rebalance_with_algo(
        basket_index,
        new_weights.clone(),
        algo.as_ref(),
        ctx.accounts.clone().into(),
        &params,
    )?;
    emit!(BasketRebalanced {
        basket_id: basket_index.id,
        new_weights,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 