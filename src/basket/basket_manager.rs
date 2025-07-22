/*!
 * Basket Manager Module
 *
 * Core basket management functionality.
 */

use crate::basket::*;
use anchor_lang::prelude::*;
use std::fmt::Debug;

/// 可插拔篮子管理引擎，支持流动性聚合与风控注入
pub trait BasketEngine: Send + Sync + Debug {
    fn create_basket(&mut self, params: BasketCreationParams) -> BasketResult<BasketInfo>;
    fn redeem_basket(&mut self, params: BasketRedemptionParams) -> BasketResult<BasketInfo>;
    fn rebalance_basket(&mut self, params: BasketRebalanceParams) -> BasketResult<BasketInfo>;
    fn get_basket_info(&self, basket_id: u64) -> BasketResult<BasketInfo>;
}

/// 篮子管理门面，支持注入不同BasketEngine、LiquidityAggregator、RiskManager实现
pub struct BasketManager<E: BasketEngine, L: LiquidityAggregator, R: RiskManager> {
    engine: E,
    liquidity: L,
    risk: R,
}

impl<E: BasketEngine, L: LiquidityAggregator, R: RiskManager> BasketManager<E, L, R> {
    pub fn new(engine: E, liquidity: L, risk: R) -> Self {
        Self {
            engine,
            liquidity,
            risk,
        }
    }

    pub fn create_basket(&mut self, params: BasketCreationParams) -> BasketResult<BasketInfo> {
        // 参数校验
        if params.token_mints.is_empty() || params.token_weights.is_empty() {
            return Err(BasketError::InvalidParameters);
        }
        if params.token_mints.len() != params.token_weights.len() {
            return Err(BasketError::InvalidParameters);
        }
        if params.token_weights.iter().any(|&w| w == 0) {
            return Err(BasketError::InvalidParameters);
        }
        // 权重归一化校验
        let total_weight: u64 = params.token_weights.iter().sum();
        if total_weight == 0 {
            return Err(BasketError::InvalidParameters);
        }
        // 风控检查
        let risk_params = RiskCheckParams {
            basket_id: 0,     // 新建时为0
            risk_score: 1000, // TODO: 真实风险评估
        };
        self.risk
            .check_risk_limits(risk_params)
            .map_err(|_| BasketError::RiskLimit)?;
        // 流动性检查
        for mint in &params.token_mints {
            let liq = self
                .liquidity
                .get_liquidity_info(*mint)
                .map_err(|_| BasketError::InternalError)?;
            if liq.total_liquidity == 0 {
                return Err(BasketError::InternalError);
            }
        }
        // 资产管理算法（示例：mint新篮子）
        let basket_id = rand::random::<u64>();
        let info = BasketInfo {
            basket_id,
            total_supply: 0,
            constituents: params.token_mints.clone(),
        };
        // 日志
        msg!(
            "Basket created: id={}, mints={:?}",
            basket_id,
            params.token_mints
        );
        Ok(info)
    }

    pub fn redeem_basket(&mut self, params: BasketRedemptionParams) -> BasketResult<BasketInfo> {
        if params.basket_id == 0 || params.amount == 0 {
            return Err(BasketError::InvalidParameters);
        }
        // 风控检查
        let risk_params = RiskCheckParams {
            basket_id: params.basket_id,
            risk_score: 1000, // TODO: 真实风险评估
        };
        self.risk
            .check_risk_limits(risk_params)
            .map_err(|_| BasketError::RiskLimit)?;
        // 资产赎回算法（示例）
        let info = BasketInfo {
            basket_id: params.basket_id,
            total_supply: 0,
            constituents: vec![],
        };
        msg!(
            "Basket redeemed: id={}, amount={}",
            params.basket_id,
            params.amount
        );
        Ok(info)
    }

    pub fn rebalance_basket(&mut self, params: BasketRebalanceParams) -> BasketResult<BasketInfo> {
        if params.basket_id == 0 || params.new_weights.is_empty() {
            return Err(BasketError::InvalidParameters);
        }
        // 权重归一化校验
        let total_weight: u64 = params.new_weights.iter().sum();
        if total_weight == 0 {
            return Err(BasketError::InvalidParameters);
        }
        // 风控检查
        let risk_params = RiskCheckParams {
            basket_id: params.basket_id,
            risk_score: 1000, // TODO: 真实风险评估
        };
        self.risk
            .check_risk_limits(risk_params)
            .map_err(|_| BasketError::RiskLimit)?;
        // 资产再平衡算法（示例）
        let info = BasketInfo {
            basket_id: params.basket_id,
            total_supply: 0,
            constituents: vec![],
        };
        msg!(
            "Basket rebalanced: id={}, new_weights={:?}",
            params.basket_id,
            params.new_weights
        );
        Ok(info)
    }

    pub fn get_basket_info(&self, basket_id: u64) -> BasketResult<BasketInfo> {
        if basket_id == 0 {
            return Err(BasketError::NotFound);
        }
        self.engine.get_basket_info(basket_id)
    }
}

/// 默认实现工厂函数
pub fn create_basket_manager(
) -> BasketManager<DefaultBasketEngine, DefaultLiquidityAggregator, DefaultRiskManager> {
    BasketManager::new(
        DefaultBasketEngine {
            history: vec![],
            ai_score: None,
        },
        DefaultLiquidityAggregator {
            history: vec![],
            ai_score: None,
        },
        DefaultRiskManager {
            history: vec![],
            ai_score: None,
        },
    )
}

/// 创建篮子参数
#[derive(Debug, Clone)]
pub struct BasketCreationParams {
    pub token_mints: Vec<Pubkey>,
    pub token_weights: Vec<u64>,
}

/// 赎回篮子参数
#[derive(Debug, Clone)]
pub struct BasketRedemptionParams {
    pub basket_id: u64,
    pub amount: u64,
}

/// 再平衡篮子参数
#[derive(Debug, Clone)]
pub struct BasketRebalanceParams {
    pub basket_id: u64,
    pub new_weights: Vec<u64>,
}

/// 篮子信息
#[derive(Debug, Clone)]
pub struct BasketInfo {
    pub basket_id: u64,
    pub total_supply: u64,
    pub constituents: Vec<Pubkey>,
}

/// 篮子相关错误
#[derive(Debug, Clone)]
pub enum BasketError {
    InvalidParameters,
    NotFound,
    Unauthorized,
    RiskLimit,
    InternalError,
}

pub type BasketResult<T> = std::result::Result<T, BasketError>;

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_basket_manager_create() {
        let mut manager = create_basket_manager();
        let params = BasketCreationParams {
            token_mints: vec![Pubkey::default()],
            token_weights: vec![10000],
        };
        let result = manager.create_basket(params);
        assert!(result.is_ok());
    }
    #[test]
    fn test_basket_manager_invalid() {
        let mut manager = create_basket_manager();
        let params = BasketCreationParams {
            token_mints: vec![],
            token_weights: vec![],
        };
        let result = manager.create_basket(params);
        assert!(result.is_err());
    }
}
