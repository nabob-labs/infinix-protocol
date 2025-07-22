/*!
 * Risk Manager Module
 *
 * Comprehensive risk management and assessment.
 */

use crate::basket::*;
use crate::core::constants::{BASIS_POINTS_MAX, MAX_CONCENTRATION_BPS, MAX_RISK_TOLERANCE_BPS};
use crate::state::optimizers::*;
use anchor_lang::prelude::*;
use std::fmt::Debug;

/// 风控引擎trait，支持多种风险评估策略
pub trait RiskManager: Send + Sync + Debug {
    fn check_risk(&mut self, params: RiskCheckParams) -> RiskResult<RiskAssessment>;
    fn get_risk_info(&self, basket_id: u64) -> RiskResult<RiskAssessment>;
}

/// 多维风险评估参数
#[derive(Debug, Clone)]
pub struct RiskCheckParams {
    pub basket_id: u64,
    pub token_weights: Vec<u64>,
    pub token_liquidity: Vec<u64>,
    pub volatility: u32,
    pub total_value: u64,
}

/// 风控门面，支持注入不同实现
pub struct RiskManagerEngine<E: RiskManager> {
    engine: E,
}

impl<E: RiskManager> RiskManagerEngine<E> {
    pub fn new(engine: E) -> Self {
        Self { engine }
    }
    pub fn check_risk(&mut self, params: RiskCheckParams) -> RiskResult<RiskAssessment> {
        self.engine.check_risk(params)
    }
    pub fn get_risk_info(&self, basket_id: u64) -> RiskResult<RiskAssessment> {
        self.engine.get_risk_info(basket_id)
    }
}

/// 默认实现（可插拔）
#[derive(Debug, Default)]
pub struct DefaultRiskManager {
    pub history: Vec<RiskAssessment>,
    pub ai_score: Option<f64>,
}

impl RiskManager for DefaultRiskManager {
    fn check_risk(&mut self, params: RiskCheckParams) -> RiskResult<RiskAssessment> {
        // 流动性风险：最小流动性 < 阈值
        let min_liquidity = params.token_liquidity.iter().min().cloned().unwrap_or(0);
        let liquidity_risk = if min_liquidity < 10_000 { 8000 } else { 2000 };
        // 集中度风险：最大权重 > 阈值
        let max_weight = params.token_weights.iter().max().cloned().unwrap_or(0);
        let concentration_risk = if max_weight > MAX_CONCENTRATION_BPS as u64 {
            7000
        } else {
            2000
        };
        // 执行风险：总价值过大
        let execution_risk = if params.total_value > 1_000_000_000 {
            6000
        } else {
            2000
        };
        // 市场风险：波动率高
        let market_risk = if params.volatility > 500 { 9000 } else { 2000 };
        // AI/ML预测（占位）
        let ai_score = self.ai_score.unwrap_or(500.0);
        // 综合风险分数
        let overall_risk_score = ((0.3 * liquidity_risk as f64
            + 0.2 * concentration_risk as f64
            + 0.2 * execution_risk as f64
            + 0.2 * market_risk as f64
            + 0.1 * ai_score) as u32)
            .min(10_000);
        let assessment = RiskAssessment {
            overall_risk_score,
            liquidity_risk,
            concentration_risk,
            execution_risk,
            market_risk,
            recommendations: vec![
                if liquidity_risk > 5000 {
                    "提升流动性".to_string()
                } else {
                    "流动性充足".to_string()
                },
                if concentration_risk > 5000 {
                    "降低集中度".to_string()
                } else {
                    "集中度合理".to_string()
                },
                if execution_risk > 5000 {
                    "分批执行".to_string()
                } else {
                    "执行风险可控".to_string()
                },
                if market_risk > 5000 {
                    "规避高波动".to_string()
                } else {
                    "市场风险可控".to_string()
                },
            ],
        };
        // 日志
        msg!(
            "Risk assessment: overall={}, liquidity={}, concentration={}, execution={}, market={}",
            overall_risk_score,
            liquidity_risk,
            concentration_risk,
            execution_risk,
            market_risk
        );
        self.history.push(assessment.clone());
        if overall_risk_score > 8000 {
            return Err(RiskError::RiskLimitExceeded);
        }
        Ok(assessment)
    }
    fn get_risk_info(&self, basket_id: u64) -> RiskResult<RiskAssessment> {
        self.history
            .iter()
            .find(|r| r.overall_risk_score == basket_id as u32)
            .cloned()
            .ok_or(RiskError::NotFound)
    }
}

/// 工厂函数：创建默认风控引擎
pub fn create_risk_manager_engine() -> RiskManagerEngine<DefaultRiskManager> {
    RiskManagerEngine::new(DefaultRiskManager::default())
}

/// 风控相关错误
#[derive(Debug, Clone)]
pub enum RiskError {
    RiskLimitExceeded,
    NotFound,
    Unauthorized,
    InternalError,
}

pub type RiskResult<T> = std::result::Result<T, RiskError>;

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_manager_check() {
        let mut manager = create_risk_manager_engine();
        let params = RiskCheckParams {
            basket_id: 1,
            token_weights: vec![5000, 3000, 2000],
            token_liquidity: vec![20_000, 15_000, 30_000],
            volatility: 200,
            total_value: 500_000_000,
        };
        let result = manager.check_risk(params);
        assert!(result.is_ok());
        let assessment = result.unwrap();
        assert!(assessment.overall_risk_score < 8000);
    }
    #[test]
    fn test_risk_manager_limit_exceeded() {
        let mut manager = create_risk_manager_engine();
        let params = RiskCheckParams {
            basket_id: 1,
            token_weights: vec![9000, 1000],
            token_liquidity: vec![1000, 2000],
            volatility: 800,
            total_value: 2_000_000_000,
        };
        let result = manager.check_risk(params);
        assert!(result.is_err());
    }
}
