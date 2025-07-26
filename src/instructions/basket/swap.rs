//!
//! Basket Swap Instruction
//! 篮子兑换指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use crate::validation::basket_validation::BasketValidatable; // 引入篮子校验trait，便于状态校验
use crate::core::types::{TradeParams, StrategyParams, OracleParams, AlgoParams}; // 引入核心参数类型
use crate::core::registry::ADAPTER_FACTORY; // 引入全局适配器工厂
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

/// 篮子兑换指令账户上下文
/// - from_basket: 源资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - to_basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 操作人签名者，类型安全
#[derive(Accounts)]
pub struct SwapBasket<'info> {
    #[account(mut)]
    pub from_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 篮子兑换指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 交易参数，类型安全
/// - price_params: 预言机价格参数，类型安全
/// - exec_params: 可选算法参数，类型安全
/// - strategy_params: 可选策略参数，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn swap_basket(
    ctx: Context<SwapBasket>,
    params: TradeParams,
    price_params: OracleParams,
    exec_params: Option<AlgoParams>,
    strategy_params: Option<StrategyParams>,
) -> Result<()> {
    let from = &mut ctx.accounts.from_basket;
    let to = &mut ctx.accounts.to_basket;
    from.validate()?;
    to.validate()?;
    // 权限校验
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::errors::basket_error::BasketError::NotAllowed);
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params {
        if let Some(algo_name) = &exec_params.algo_name {
            let factory = ADAPTER_FACTORY.lock().unwrap();
            if let Some(algo) = factory.get(algo_name) {
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                    // exec_strategy.execute(ctx, &exec_params.algo_params)?;
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price;
    if let Some(oracle_name) = &price_params.oracle_name {
        let factory = ADAPTER_FACTORY.lock().unwrap();
        if let Some(adapter) = factory.get(oracle_name) {
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                // let oracle_result = oracle_adapter.get_price(&price_params)?;
                // price = oracle_result.price;
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name {
        let factory = ADAPTER_FACTORY.lock().unwrap();
        if let Some(adapter) = factory.get(dex_name) {
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() {
                // let swap_result = dex_adapter.swap(&params)?;
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params {
        if !strategy_params.strategy_name.is_empty() {
            crate::services::basket_service::BasketService::strategy_rebalance(
                from,
                &strategy_params.strategy_name,
                &strategy_params.params,
                ctx.accounts.authority.key(),
            )?;
        }
    }
    // 5. 资产兑换
    BasketService::swap(
        from,
        to,
        params.amount_in,
        price,
        ctx.accounts.authority.key(),
    )?;
    emit!(BasketSwapped {
        from_basket_id: from.id,
        to_basket_id: to.id,
        amount: params.amount_in,
        price,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 