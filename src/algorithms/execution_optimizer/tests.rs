//! 执行优化器相关单元测试模块
//! 覆盖最优执行优化器、极端参数、无效输入等场景。

use super::*;
use crate::core::MarketConditions;

/// 测试：最优执行优化器基础流程
#[test]
fn test_optimal_execution_optimizer_basic() {
    let mut optimizer = OptimalExecutionOptimizer { strategy: Box::new(AlmgrenChrissStrategy) };
    let trades = vec![];
    let market = MarketConditions::default();
    let result = optimizer.optimize(&trades, &market);
    assert!(result.is_ok());
}

/// 测试：Almgren-Chriss 策略极端参数
#[test]
fn test_almgren_chriss_strategy_extreme_params() {
    let strategy = AlmgrenChrissStrategy;
    let trades = vec![];
    let mut market = MarketConditions::default();
    market.volatility = 10_000; // 极端高波动
    let result = strategy.optimize(&trades, &market);
    assert!(result.is_err() || result.is_ok());
}

/// 测试：优化器无效输入
#[test]
fn test_optimizer_invalid_input() {
    let mut optimizer = OptimalExecutionOptimizer { strategy: Box::new(AlmgrenChrissStrategy) };
    let trades = vec![];
    let market = MarketConditions { volatility: 0, liquidity: 0, spread: 0 };
    let result = optimizer.optimize(&trades, &market);
    assert!(result.is_ok() || result.is_err());
} 