/*!
 * Trading Engine Module
 *
 * Core trading engine for basket operations.
 */

use crate::basket::*;
use anchor_lang::prelude::*;
use std::fmt::Debug;

/// 交易执行引擎trait，支持多类型可插拔实现
pub trait TradingEngine: Send + Sync + Debug {
    fn execute_trade(&mut self, params: TradeExecutionParams) -> TradeResult<TradeExecutionInfo>;
    fn get_trade_info(&self, trade_id: u64) -> TradeResult<TradeExecutionInfo>;
}

/// 市价单交易引擎
#[derive(Debug, Default)]
pub struct MarketOrderEngine;

impl TradingEngine for MarketOrderEngine {
    fn execute_trade(&mut self, params: TradeExecutionParams) -> TradeResult<TradeExecutionInfo> {
        if params.amount == 0 || params.token_mint == Pubkey::default() {
            return Err(TradeError::InvalidParameters);
        }
        // 模拟市价单撮合，滑点与成本动态建模
        let execution_cost = params.amount * 1000 / 1_000_000; // 假设1u = 0.001 token
        let slippage = 50 + (params.amount / 10_000); // 动态滑点
        msg!(
            "Market order executed: amount={}, cost={}, slippage={}",
            params.amount,
            execution_cost,
            slippage
        );
        Ok(TradeExecutionInfo {
            trade_id: rand::random::<u64>(),
            amount: params.amount,
            token_mint: params.token_mint,
            execution_cost,
            slippage,
            success: true,
        })
    }
    fn get_trade_info(&self, trade_id: u64) -> TradeResult<TradeExecutionInfo> {
        if trade_id == 0 {
            return Err(TradeError::NotFound);
        }
        Ok(TradeExecutionInfo {
            trade_id,
            amount: 1000,
            token_mint: Pubkey::default(),
            execution_cost: 1000,
            slippage: 50,
            success: true,
        })
    }
}

/// 限价单交易引擎（示例）
#[derive(Debug, Default)]
pub struct LimitOrderEngine;

impl TradingEngine for LimitOrderEngine {
    fn execute_trade(&mut self, params: TradeExecutionParams) -> TradeResult<TradeExecutionInfo> {
        if params.amount == 0 || params.token_mint == Pubkey::default() {
            return Err(TradeError::InvalidParameters);
        }
        // 模拟限价单撮合，假设部分成交
        let execution_cost = params.amount * 999 / 1_000_000;
        let slippage = 20;
        msg!(
            "Limit order executed: amount={}, cost={}, slippage={}",
            params.amount,
            execution_cost,
            slippage
        );
        Ok(TradeExecutionInfo {
            trade_id: rand::random::<u64>(),
            amount: params.amount,
            token_mint: params.token_mint,
            execution_cost,
            slippage,
            success: true,
        })
    }
    fn get_trade_info(&self, trade_id: u64) -> TradeResult<TradeExecutionInfo> {
        if trade_id == 0 {
            return Err(TradeError::NotFound);
        }
        Ok(TradeExecutionInfo {
            trade_id,
            amount: 1000,
            token_mint: Pubkey::default(),
            execution_cost: 999,
            slippage: 20,
            success: true,
        })
    }
}

/// TWAP/VWAP等高级交易引擎可通过组合已有算法模块实现
/// 交易执行管理器，支持注入不同TradingEngine实现
pub struct TradingEngineManager<E: TradingEngine> {
    engine: E,
}

impl<E: TradingEngine> TradingEngineManager<E> {
    pub fn new(engine: E) -> Self {
        Self { engine }
    }
    pub fn execute_trade(
        &mut self,
        params: TradeExecutionParams,
    ) -> TradeResult<TradeExecutionInfo> {
        self.engine.execute_trade(params)
    }
    pub fn get_trade_info(&self, trade_id: u64) -> TradeResult<TradeExecutionInfo> {
        self.engine.get_trade_info(trade_id)
    }
}

/// 默认实现工厂函数（市价单）
pub fn create_trading_engine_manager() -> TradingEngineManager<MarketOrderEngine> {
    TradingEngineManager::new(MarketOrderEngine::default())
}

/// 交易执行参数
#[derive(Debug, Clone)]
pub struct TradeExecutionParams {
    pub amount: u64,
    pub token_mint: Pubkey,
}

/// 交易执行信息
#[derive(Debug, Clone)]
pub struct TradeExecutionInfo {
    pub trade_id: u64,
    pub amount: u64,
    pub token_mint: Pubkey,
    pub execution_cost: u64,
    pub slippage: u64,
    pub success: bool,
}

/// 交易相关错误
#[derive(Debug, Clone)]
pub enum TradeError {
    InvalidParameters,
    NotFound,
    Unauthorized,
    RiskLimit,
    InternalError,
}

pub type TradeResult<T> = std::result::Result<T, TradeError>;

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_market_order_execute() {
        let mut manager = create_trading_engine_manager();
        let params = TradeExecutionParams {
            amount: 1000,
            token_mint: Pubkey::new_unique(),
        };
        let result = manager.execute_trade(params);
        assert!(result.is_ok());
    }
    #[test]
    fn test_market_order_invalid() {
        let mut manager = create_trading_engine_manager();
        let params = TradeExecutionParams {
            amount: 0,
            token_mint: Pubkey::default(),
        };
        let result = manager.execute_trade(params);
        assert!(result.is_err());
    }
}

/// Basket trade definition
#[derive(Debug, Clone)]
pub struct BasketTrade {
    pub trade_type: TradeType,
    pub amount: u64,
    pub composition: BasketComposition,
}
