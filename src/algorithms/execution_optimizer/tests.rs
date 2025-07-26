//! 执行优化器相关单元测试模块
//! 覆盖最优执行优化器、极端参数、无效输入等场景。

use super::*;
use crate::algorithms::execution_optimizer::{GeneticOptimizer, MlOptimizer, OptimizationParams};

/// 测试：GeneticOptimizer基础功能
#[test]
fn test_genetic_optimizer_basic() {
    // 构造典型参数
    let params = OptimizationParams {
        order_size: 100,
        market_impact: 5,
        slippage_tolerance: 2,
        ..Default::default()
    };
    let optimizer = GeneticOptimizer::default();
    let result = optimizer.optimize(&params);
    // 断言优化值大于0且不超过order_size*2
    assert!(result.optimized_value > 0 && result.optimized_value <= 200, "Genetic优化值异常");
}

/// 测试：GeneticOptimizer极端参数
#[test]
fn test_genetic_optimizer_extreme() {
    // 极端order_size=0
    let params = OptimizationParams {
        order_size: 0,
        market_impact: 0,
        slippage_tolerance: 0,
        ..Default::default()
    };
    let optimizer = GeneticOptimizer::default();
    let result = optimizer.optimize(&params);
    // 断言优化值为1（最小值）
    assert_eq!(result.optimized_value, 1, "极端参数下Genetic优化值应为1");
}

/// 测试：MlOptimizer基础功能
#[test]
fn test_ml_optimizer_basic() {
    let params = OptimizationParams {
        order_size: 100,
        market_impact: 5,
        slippage_tolerance: 2,
        ..Default::default()
    };
    let optimizer = MlOptimizer::default();
    let result = optimizer.optimize(&params);
    // 断言优化值大于0
    assert!(result.optimized_value > 0, "ML优化值应大于0");
}

/// 测试：MlOptimizer极端参数
#[test]
fn test_ml_optimizer_extreme() {
    let params = OptimizationParams {
        order_size: 0,
        market_impact: 0,
        slippage_tolerance: 0,
        ..Default::default()
    };
    let optimizer = MlOptimizer::default();
    let result = optimizer.optimize(&params);
    // 断言优化值为1（归一化后预测最小值）
    assert_eq!(result.optimized_value, 1, "极端参数下ML优化值应为1");
} 