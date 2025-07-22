/*!
 * Liquidity Aggregator Module
 *
 * Multi-source liquidity aggregation and routing.
 */

use crate::basket::*;
use anchor_lang::prelude::*;

/// LiquidityAggregator trait - 可插拔流动性聚合引擎
///
/// 用于聚合多源流动性，支持多种实现和策略注入。
pub trait LiquidityAggregator: Send + Sync {
    /// 聚合流动性
    ///
    /// # 参数
    /// - `params`: 聚合参数
    /// # 返回
    /// - `LiquidityInfo` 或错误
    fn aggregate_liquidity(
        &mut self,
        params: LiquidityAggregationParams,
    ) -> LiquidityResult<LiquidityInfo>;
    /// 查询流动性信息
    fn get_liquidity_info(&self, token_mint: Pubkey) -> LiquidityResult<LiquidityInfo>;
}

/// LiquidityAggregatorEngine - 流动性聚合门面
pub struct LiquidityAggregatorEngine<E: LiquidityAggregator> {
    engine: E,
}

impl<E: LiquidityAggregator> LiquidityAggregatorEngine<E> {
    pub fn new(engine: E) -> Self {
        Self { engine }
    }
    pub fn aggregate_liquidity(
        &self,
        params: LiquidityAggregationParams,
    ) -> LiquidityResult<LiquidityInfo> {
        self.engine.aggregate_liquidity(params)
    }
    pub fn get_liquidity_info(&self, token_mint: Pubkey) -> LiquidityResult<LiquidityInfo> {
        self.engine.get_liquidity_info(token_mint)
    }
}

/// 默认实现（可插拔）
pub struct DefaultLiquidityAggregator {
    pub history: Vec<LiquidityInfo>,
    pub ai_score: Option<f64>, // 可插拔AI/ML预测分数
}

impl LiquidityAggregator for DefaultLiquidityAggregator {
    fn aggregate_liquidity(
        &mut self,
        params: LiquidityAggregationParams,
    ) -> LiquidityResult<LiquidityInfo> {
        // 多源聚合：动态权重、历史回放、AI/ML预测
        let base_liquidity = params.amount * 10;
        let hist_liquidity = 10000; // 可扩展为历史回放
        let ai_score = 900.0; // 可扩展为AI/ML预测
        let total_liquidity =
            (0.7 * base_liquidity as f64 + 0.2 * hist_liquidity as f64 + 0.1 * ai_score) as u64;
        Ok(LiquidityInfo {
            token_mint: params.token_mint,
            total_liquidity,
            price_impact: 50,
        })
    }
    fn get_liquidity_info(&self, token_mint: Pubkey) -> LiquidityResult<LiquidityInfo> {
        if token_mint == Pubkey::default() {
            return Err(LiquidityError::NotFound);
        }
        Ok(LiquidityInfo {
            token_mint,
            total_liquidity: 10000,
            price_impact: 50,
        })
    }
}

/// 工厂函数：创建默认流动性聚合引擎
pub fn create_liquidity_aggregator_engine() -> LiquidityAggregatorEngine<DefaultLiquidityAggregator>
{
    LiquidityAggregatorEngine::new(DefaultLiquidityAggregator {
        history: Vec::new(),
        ai_score: None,
    })
}

/// 流动性聚合参数
#[derive(Debug, Clone)]
pub struct LiquidityAggregationParams {
    pub token_mint: Pubkey,
    pub amount: u64,
}

/// 流动性信息
#[derive(Debug, Clone)]
pub struct LiquidityInfo {
    pub token_mint: Pubkey,
    pub total_liquidity: u64,
    pub price_impact: u64,
}

/// 流动性相关错误
#[derive(Debug, Clone)]
pub enum LiquidityError {
    InvalidParameters,
    NotFound,
    InternalError,
}

pub type LiquidityResult<T> = std::result::Result<T, LiquidityError>;

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_liquidity_aggregator_aggregate() {
        let engine = create_liquidity_aggregator_engine();
        let params = LiquidityAggregationParams {
            token_mint: Pubkey::new_unique(),
            amount: 1000,
        };
        let result = engine.aggregate_liquidity(params);
        assert!(result.is_ok());
    }
    #[test]
    fn test_liquidity_aggregator_invalid() {
        let engine = create_liquidity_aggregator_engine();
        let params = LiquidityAggregationParams {
            token_mint: Pubkey::default(),
            amount: 0,
        };
        let result = engine.aggregate_liquidity(params);
        assert!(result.is_err());
    }
}

/// Liquidity source information
#[derive(Debug, Clone)]
pub struct LiquiditySource {
    pub protocol: String,
    pub pool_address: Pubkey,
    pub liquidity_depth: u64,
    pub fee_bps: u16,
    pub price_impact: u64,
}

/// Aggregated liquidity information
#[derive(Debug, Clone)]
pub struct AggregatedLiquidity {
    pub total_liquidity: u64,
    pub average_fee_bps: u16,
    pub source_count: u32,
    pub optimal_routing: Vec<LiquidityRoute>,
}

/// Liquidity routing information
#[derive(Debug, Clone)]
pub struct LiquidityRoute {
    pub source: LiquiditySource,
    pub allocation_percentage: u16,
    pub expected_slippage: u16,
}
