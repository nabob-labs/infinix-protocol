/*!
 * Execution Optimizer Module
 *
 * AI-powered execution optimization algorithms.
 */

use crate::algorithms::{twap, vwap};
use crate::basket::ExecutionStrategy;
use crate::core::constants::{DEFAULT_CACHE_TTL, MAX_SLIPPAGE_BPS};
use crate::core::*;
use crate::state::optimizers::*;
use anchor_lang::prelude::*;
use std::fmt::Debug;

/// 执行优化器trait，支持多种算法实现
pub trait ExecutionOptimizer: Send + Sync + Debug {
    fn optimize(
        &mut self,
        trades: &[BasketTrade],
        market: &MarketConditions,
    ) -> StrategyResult<ExecutionStrategy>;
}

/// TWAP执行优化器实现
#[derive(Debug, Default)]
pub struct TwapExecutionOptimizer;

impl ExecutionOptimizer for TwapExecutionOptimizer {
    fn optimize(
        &mut self,
        trades: &[BasketTrade],
        market: &MarketConditions,
    ) -> StrategyResult<ExecutionStrategy> {
        // 仅示例：将所有trade合并为一个TWAP计划
        let total_amount: u64 = trades.iter().map(|t| t.amount).sum();
        let token_mint = trades.get(0).map(|t| t.mint).unwrap_or(Pubkey::default());
        let twap_input = twap::TwapInput {
            total_amount,
            token_mint,
        };
        let twap_config = twap::TwapConfig::default();
        let mut market_data = twap::EnhancedMarketData::default();
        // 可扩展：填充market_data
        let mut algo = twap::TwapAlgorithm::new();
        let output = algo.execute(twap_input, &twap_config, &market_data)?;
        Ok(ExecutionStrategy {
            execution_type: ExecutionType::TWAP,
            batch_size: output.schedule.len() as u64,
            time_horizon: output.estimated_duration_ms as i64 / 1000,
            slippage_tolerance: 100,
            constituent_orders: vec![], // 可扩展为详细订单
        })
    }
}

/// VWAP执行优化器实现
#[derive(Debug, Default)]
pub struct VwapExecutionOptimizer;

impl ExecutionOptimizer for VwapExecutionOptimizer {
    fn optimize(
        &mut self,
        trades: &[BasketTrade],
        market: &MarketConditions,
    ) -> StrategyResult<ExecutionStrategy> {
        let total_amount: u64 = trades.iter().map(|t| t.amount).sum();
        let token_mint = trades.get(0).map(|t| t.mint).unwrap_or(Pubkey::default());
        let vwap_input = vwap::VwapInput {
            total_amount,
            token_mint,
        };
        let vwap_config = vwap::VwapConfig {
            duration_seconds: 3600,
            interval_seconds: 300,
            lookback_seconds: 3600,
        };
        let mut market_data = vwap::EnhancedMarketData::default();
        // 可扩展：填充market_data
        let mut algo = vwap::VwapAlgorithm::new();
        let output = algo.execute(vwap_input, &vwap_config, &market_data)?;
        Ok(ExecutionStrategy {
            execution_type: ExecutionType::VWAP,
            batch_size: output.schedule.len() as u64,
            time_horizon: (output.schedule.len() * 300) as i64,
            slippage_tolerance: 100,
            constituent_orders: vec![],
        })
    }
}

/// Optimal执行优化器实现（可扩展为AI/ML/历史回放等）
#[derive(Debug, Default)]
pub struct OptimalExecutionOptimizer;

impl ExecutionOptimizer for OptimalExecutionOptimizer {
    fn optimize(
        &mut self,
        trades: &[BasketTrade],
        market: &MarketConditions,
    ) -> StrategyResult<ExecutionStrategy> {
        // TODO: 实现更复杂的最优执行算法（如Almgren-Chriss、AI/ML等）
        // 目前占位，选择TWAP或VWAP中得分更高者
        let mut twap = TwapExecutionOptimizer::default();
        let mut vwap = VwapExecutionOptimizer::default();
        let twap_result = twap.optimize(trades, market);
        let vwap_result = vwap.optimize(trades, market);
        // 简单比较，后续可用AI/ML/历史回放等加权
        match (twap_result, vwap_result) {
            (Ok(t), Ok(v)) => {
                if t.batch_size <= v.batch_size {
                    Ok(t)
                } else {
                    Ok(v)
                }
            }
            (Ok(t), Err(_)) => Ok(t),
            (Err(_), Ok(v)) => Ok(v),
            (Err(e), Err(_)) => Err(e),
        }
    }
}

/// 工厂函数：创建默认执行优化器（可配置）
pub fn create_execution_optimizer(typ: ExecutionType) -> Box<dyn ExecutionOptimizer> {
    match typ {
        ExecutionType::TWAP => Box::new(TwapExecutionOptimizer::default()),
        ExecutionType::VWAP => Box::new(VwapExecutionOptimizer::default()),
        ExecutionType::Optimal => Box::new(OptimalExecutionOptimizer::default()),
        _ => Box::new(TwapExecutionOptimizer::default()),
    }
}

// 兼容原有接口，保留历史/AI分数等字段
#[derive(Debug, Default)]
pub struct ExecutionOptimizerImpl {
    pub history: Vec<ExecutionStrategy>,
    pub ai_score: Option<f64>,
    pub optimizer: Box<dyn ExecutionOptimizer>,
}

impl ExecutionOptimizerImpl {
    pub fn new(typ: ExecutionType) -> Self {
        Self {
            history: vec![],
            ai_score: None,
            optimizer: create_execution_optimizer(typ),
        }
    }
    pub fn optimize_execution(
        &mut self,
        trades: &[BasketTrade],
        market: &MarketConditions,
    ) -> StrategyResult<ExecutionStrategy> {
        let result = self.optimizer.optimize(trades, market);
        if let Ok(ref strat) = result {
            self.history.push(strat.clone());
        }
        result
    }
    pub fn calculate_optimization_score(
        original_cost: u64,
        optimized_cost: u64,
        original_time: u64,
        optimized_time: u64,
    ) -> u32 {
        let cost_improvement = if original_cost > optimized_cost {
            ((original_cost - optimized_cost) * 10000) / original_cost
        } else {
            0
        };
        let time_improvement = if original_time > optimized_time {
            ((original_time - optimized_time) * 10000) / original_time
        } else {
            0
        };
        ((cost_improvement + time_improvement) / 2) as u32
    }
}
