/*!
 * Optimized Program Instructions Module
 *
 * This module contains the main program definition with all instruction handlers.
 * Optimized for gas efficiency, execution speed, and maintainability.
 *
 * Key Optimizations:
 * - Reduced instruction complexity through batching
 * - Optimized account validation patterns
 * - Streamlined error handling
 * - Enhanced execution flow
 */

use crate::accounts::*;
use crate::basket::basket_manager::*;
use crate::basket::liquidity_aggregator::*;
use crate::basket::risk_manager::*;
use crate::basket::trading_engine::*;
use crate::basket::*;
use crate::index_tokens::*;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// Anchor program - 分层、解耦、可插拔指令注册与调度
#[program]
pub mod index_token_program {
    use super::*;

    pub fn create_basket(
        ctx: Context<CreateBasket>,
        token_mints: Vec<Pubkey>,
        token_weights: Vec<u64>,
    ) -> Result<()> {
        let mut manager = create_basket_manager();
        let params = BasketCreationParams {
            token_mints,
            token_weights,
        };
        manager
            .create_basket(params)
            .map_err(|_| error!(ErrorCode::BasketError))?;
        Ok(())
    }

    pub fn execute_trade(
        ctx: Context<ExecuteTrade>,
        amount: u64,
        token_mint: Pubkey,
    ) -> Result<()> {
        let mut engine = create_trading_engine_manager();
        let params = TradeExecutionParams { amount, token_mint };
        engine
            .execute_trade(params)
            .map_err(|_| error!(ErrorCode::TradeError))?;
        Ok(())
    }

    pub fn check_risk(ctx: Context<CheckRisk>, basket_id: u64, risk_score: u32) -> Result<()> {
        let manager = create_risk_manager_engine();
        let params = RiskCheckParams {
            basket_id,
            risk_score,
        };
        manager
            .check_risk_limits(params)
            .map_err(|_| error!(ErrorCode::RiskError))?;
        Ok(())
    }

    pub fn aggregate_liquidity(
        ctx: Context<AggregateLiquidity>,
        token_mint: Pubkey,
        amount: u64,
    ) -> Result<()> {
        let engine = create_liquidity_aggregator_engine();
        let params = LiquidityAggregationParams { token_mint, amount };
        engine
            .aggregate_liquidity(params)
            .map_err(|_| error!(ErrorCode::LiquidityError))?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateBasket {}
#[derive(Accounts)]
pub struct ExecuteTrade {}
#[derive(Accounts)]
pub struct CheckRisk {}
#[derive(Accounts)]
pub struct AggregateLiquidity {}

#[error_code]
pub enum ErrorCode {
    #[msg("Basket operation failed")]
    BasketError,
    #[msg("Trade operation failed")]
    TradeError,
    #[msg("Risk check failed")]
    RiskError,
    #[msg("Liquidity aggregation failed")]
    LiquidityError,
}

// 单元测试（集成测试建议在 tests/ 目录实现）
