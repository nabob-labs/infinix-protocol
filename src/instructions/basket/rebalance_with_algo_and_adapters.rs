//!
//! Basket Rebalance With Algo And Adapters Instruction
//! 篮子带算法和适配器再平衡指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketServiceFacade; // 引入篮子服务层，封装核心业务逻辑
// BasketValidatable trait not found, removing import // 引入篮子校验trait，便于状态校验
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RebalanceWithAlgoAndAdaptersParams {
    pub new_weights: Vec<u64>,
    pub algo_name: String,
    pub dex_name: String,
    pub oracle_name: String,
    pub params: crate::algorithms::traits::ExecutionParams,
}

/// 篮子带算法和适配器再平衡指令账户上下文
/// - basket_index: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct RebalanceBasketWithAlgoAndAdapters<'info> {
    #[account(mut)]
    pub basket_index: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子带算法和适配器再平衡指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - args: 结构体参数，包含新权重、算法、DEX、预言机等
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn rebalance_basket_with_algo_and_adapters(
    ctx: Context<RebalanceBasketWithAlgoAndAdapters>,
    args: RebalanceWithAlgoAndAdaptersParams,
) -> anchor_lang::Result<()> {
    use crate::algorithms::algorithm_registry::AlgorithmRegistry;
    use crate::dex::adapter::DexAdapterRegistry;
    use crate::oracles::adapter_registry::OracleAdapterRegistry;
    let algo_registry = AlgorithmRegistry::new();
    let dex_registry = DexAdapterRegistry::new();
    let oracle_registry = OracleAdapterRegistry::new();
    let algo = algo_registry
        .get_execution(&args.algo_name)
        .ok_or(crate::errors::basket_error::BasketError::Unknown)?;
    let dex = dex_registry
        .get(&args.dex_name)
        .ok_or(crate::errors::basket_error::BasketError::Unknown)?;
    let oracle = oracle_registry
        .get(&args.oracle_name)
        .ok_or(crate::errors::basket_error::BasketError::Unknown)?;
    let basket_index = &mut ctx.accounts.basket_index;
    basket_index.validate()?;
    crate::services::basket_service::rebalance_with_algo_and_adapters(
        basket_index,
        args.new_weights.clone(),
        algo.as_ref(),
        dex.as_ref(),
        oracle.as_ref(),
        ctx.accounts.clone().into(),
        &args.params,
    )?;
    emit!(crate::events::basket_event::BasketRebalanced {
        basket_id: basket_index.id,
        new_weights: args.new_weights,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 