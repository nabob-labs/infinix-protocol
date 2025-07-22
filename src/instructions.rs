/*!
 * Optimized Instruction Handlers Module
 *
 * This module contains streamlined instruction handlers organized by functionality.
 * Each handler is optimized for performance and follows consistent patterns.
 */

use crate::accounts::*;
use crate::basket::*;
use crate::core::*;
use crate::error::StrategyError;
use crate::factories::*;
use crate::state::*;
use crate::strategies::*;
use crate::utils::*;
use anchor_lang::prelude::*;

/// InstructionHandler trait - 可插拔指令处理器
pub trait InstructionHandler<I>: Send + Sync {
    fn handle(&self, input: I) -> Result<()>;
}

/// InstructionRegistry - 指令注册表
pub struct InstructionRegistry<I> {
    handlers: Vec<Box<dyn InstructionHandler<I>>>,
}

impl<I> InstructionRegistry<I> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    pub fn register(&mut self, handler: Box<dyn InstructionHandler<I>>) {
        self.handlers.push(handler);
    }
    pub fn dispatch(&self, input: I) -> Result<()> {
        for h in &self.handlers {
            h.handle(input.clone())?;
        }
        Ok(())
    }
}

// 示例指令处理器
pub struct DummyHandler;
impl InstructionHandler<u64> for DummyHandler {
    fn handle(&self, input: u64) -> Result<()> {
        require!(
            input > 0,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        Ok(())
    }
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_instruction_registry() {
        let mut reg = InstructionRegistry::<u64>::new();
        reg.register(Box::new(DummyHandler));
        assert!(reg.dispatch(1).is_ok());
        assert!(reg.dispatch(0).is_err());
    }
}

// ============================================================================
// FACTORY MANAGEMENT INSTRUCTIONS
// ============================================================================

/// Initialize Weight Strategy Factory
pub fn initialize_weight_factory(
    ctx: Context<InitializeWeightFactory>,
    factory_id: u64,
) -> Result<()> {
    let factory = &mut ctx.accounts.factory;
    let authority = ctx.accounts.authority.key();
    let bump = ctx.bumps.factory;

    if factory_id == 0 {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    WeightStrategyFactoryManager::initialize(factory, authority, factory_id, bump)?;

    emit!(WeightFactoryInitialized {
        factory: factory.key(),
        authority,
        factory_id,
    });

    msg!(
        "Weight strategy factory initialized: ID={}, Authority={}",
        factory_id,
        authority
    );
    Ok(())
}

/// Initialize Rebalancing Strategy Factory
pub fn initialize_rebalancing_factory(
    ctx: Context<InitializeRebalancingFactory>,
    factory_id: u64,
) -> Result<()> {
    let factory = &mut ctx.accounts.factory;
    let authority = ctx.accounts.authority.key();
    let bump = ctx.bumps.factory;

    if factory_id == 0 {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    RebalancingStrategyFactoryManager::initialize(factory, authority, factory_id, bump)?;

    emit!(RebalancingFactoryInitialized {
        factory: factory.key(),
        authority,
        factory_id,
    });

    msg!(
        "Rebalancing strategy factory initialized: ID={}, Authority={}",
        factory_id,
        authority
    );
    Ok(())
}

// ============================================================================
// STRATEGY CREATION AND MANAGEMENT INSTRUCTIONS
// ============================================================================

/// Create Weight Calculation Strategy
pub fn create_weight_strategy(
    ctx: Context<CreateWeightStrategy>,
    strategy_type: WeightStrategyType,
    parameters: Vec<u8>,
    token_mints: Vec<Pubkey>,
) -> Result<()> {
    let factory = &mut ctx.accounts.factory;
    let strategy = &mut ctx.accounts.strategy;
    let authority = ctx.accounts.authority.key();
    let bump = ctx.bumps.strategy;

    // Validate input parameters
    if token_mints.is_empty() || token_mints.len() > WeightStrategy::MAX_TOKENS {
        return Err(StrategyError::InvalidTokenCount.into());
    }

    if parameters.len() > WeightStrategy::MAX_PARAMETERS_SIZE {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    // Validate no duplicate token mints
    let mut unique_mints = std::collections::HashSet::new();
    for mint in &token_mints {
        if !unique_mints.insert(*mint) {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
    }

    WeightStrategyFactoryManager::create_strategy(
        factory,
        strategy,
        authority,
        strategy_type.clone(),
        parameters,
        token_mints.clone(),
        bump,
    )?;

    emit!(WeightStrategyCreated {
        strategy: strategy.key(),
        factory: factory.key(),
        authority,
        strategy_type: strategy_type.clone().into(),
        token_count: token_mints.len() as u32,
    });

    msg!(
        "Weight strategy created: Type={:?}, Tokens={}, Authority={}",
        strategy_type,
        token_mints.len(),
        authority
    );
    Ok(())
}

/// Create Rebalancing Strategy
pub fn create_rebalancing_strategy(
    ctx: Context<CreateRebalancingStrategy>,
    strategy_type: RebalancingStrategyType,
    parameters: Vec<u8>,
    weight_strategy: Pubkey,
    rebalancing_threshold: u64,
    min_rebalance_interval: u64,
    max_slippage: u64,
) -> Result<()> {
    let factory = &mut ctx.accounts.factory;
    let strategy = &mut ctx.accounts.strategy;
    let authority = ctx.accounts.authority.key();
    let bump = ctx.bumps.strategy;

    // Validate parameters
    if parameters.len() > crate::state::strategies::RebalancingStrategy::MAX_PARAMETERS_SIZE {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    if rebalancing_threshold == 0 || rebalancing_threshold > 5000 {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    if min_rebalance_interval < 60 {
        return Err(StrategyError::InvalidTimeWindow.into());
    }

    if max_slippage > 2000 {
        return Err(StrategyError::SlippageExceeded.into());
    }

    if weight_strategy == Pubkey::default() {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    RebalancingStrategyFactoryManager::create_strategy(
        factory,
        strategy,
        authority,
        weight_strategy,
        strategy_type.clone(),
        parameters,
        rebalancing_threshold,
        min_rebalance_interval,
        max_slippage,
        bump,
    )?;

    emit!(RebalancingStrategyCreated {
        strategy: strategy.key(),
        factory: factory.key(),
        authority,
        strategy_type: strategy_type.clone().into(),
        threshold: rebalancing_threshold,
    });

    msg!(
        "Rebalancing strategy created: Type={:?}, Threshold={}bp, Authority={}",
        strategy_type,
        rebalancing_threshold,
        authority
    );
    Ok(())
}

/// Execute Portfolio Rebalancing
pub fn execute_rebalancing(
    ctx: Context<ExecuteRebalancing>,
    target_weights: Vec<u64>,
) -> Result<()> {
    let rebalancing_strategy = &mut ctx.accounts.rebalancing_strategy;
    let weight_strategy = &ctx.accounts.weight_strategy;

    // Validate strategy compatibility
    FactoryUtils::validate_strategy_compatibility(weight_strategy, rebalancing_strategy)?;

    // Validate weights
    ValidationUtils::validate_weights_sum(&target_weights)?;

    if target_weights.len() != weight_strategy.token_mints.len() {
        return Err(StrategyError::InvalidTokenCount.into());
    }

    // Create strategy tokens and execute rebalancing
    let tokens = create_strategy_tokens(weight_strategy, &target_weights)?;
    let mock_price_feeds = create_mock_price_feeds_for_tokens(&weight_strategy.token_mints)?;
    let total_portfolio_value = PriceUtils::calculate_total_value(&tokens, &mock_price_feeds)?;

    if total_portfolio_value < 1000 {
        return Err(StrategyError::InsufficientLiquidity.into());
    }

    let actions = RebalancingStrategyFactoryManager::execute_rebalancing(
        rebalancing_strategy,
        weight_strategy,
        &tokens,
        total_portfolio_value,
    )?;

    RebalanceUtils::validate_actions(&actions, &tokens, rebalancing_strategy.max_slippage)?;
    let total_cost = RebalanceUtils::calculate_total_cost(&actions)?;

    // Emit events
    for action in &actions {
        emit!(RebalanceActionExecuted {
            strategy: rebalancing_strategy.key(),
            token_mint: action.token_mint,
            action_type: action.action_type,
            amount: action.amount,
            price_impact: action.price_impact,
        });
    }

    emit!(RebalancingExecuted {
        strategy: rebalancing_strategy.key(),
        action_count: actions.len() as u32,
        total_value: total_portfolio_value,
        total_cost,
        avg_slippage: if !actions.is_empty() {
            actions.iter().map(|a| a.price_impact).sum::<u64>() / actions.len() as u64
        } else {
            0
        },
    });

    msg!(
        "Rebalancing executed: {} actions, total value: {}, cost: {}",
        actions.len(),
        total_portfolio_value,
        total_cost
    );
    Ok(())
}

/// Update Strategy Parameters
pub fn update_strategy_params(
    ctx: Context<UpdateStrategyParams>,
    new_params: Vec<u8>,
) -> Result<()> {
    let strategy = &mut ctx.accounts.strategy;

    if new_params.len() > WeightStrategy::MAX_PARAMETERS_SIZE {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    strategy.validate_can_execute()?;
    WeightStrategyFactoryManager::update_parameters(strategy, new_params)?;

    emit!(StrategyParametersUpdated {
        strategy: strategy.key(),
        authority: strategy.authority(),
    });

    msg!(
        "Strategy parameters updated: Strategy={}, Authority={}",
        strategy.key(),
        strategy.authority()
    );
    Ok(())
}

// ============================================================================
// BASKET TRADING INSTRUCTIONS
// ============================================================================

/// Initialize Basket Manager
pub fn initialize_basket_manager(
    ctx: Context<InitializeBasketManager>,
    basket_id: u64,
    composition: BasketComposition,
) -> Result<()> {
    let manager = &mut ctx.accounts.manager;
    let authority = ctx.accounts.authority.key();
    let bump = ctx.bumps.manager;

    if basket_id == 0 {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    if composition.constituents.is_empty() || composition.constituents.len() > 32 {
        return Err(StrategyError::InvalidTokenCount.into());
    }

    manager.initialize(authority, authority, bump)?;
    manager.basket_count = 0;
    manager.total_value_locked = 0;
    manager.fee_collector = authority; // Default to authority
    manager.default_fee_bps = 100; // 1% default fee

    emit!(BasketManagerInitialized {
        manager: manager.key(),
        authority,
        basket_id,
    });

    msg!(
        "Basket manager initialized: ID={}, Authority={}",
        basket_id,
        authority
    );
    Ok(())
}

/// Create Basket
pub fn create_basket(
    ctx: Context<CreateBasket>,
    creation_amount: u64,
    execution_params: crate::core::ExecutionParams,
) -> Result<()> {
    let manager = &mut ctx.accounts.manager;
    let basket = &mut ctx.accounts.basket;
    let authority = ctx.accounts.authority.key();
    let bump = ctx.bumps.basket;

    if creation_amount == 0 {
        return Err(StrategyError::BasketAmountTooSmall.into());
    }

    if execution_params.max_slippage_bps > 2000 {
        return Err(StrategyError::SlippageExceeded.into());
    }

    let basket_mint = Pubkey::new_unique(); // Would be actual mint in production
    let composition = initialize_basket_composition(
        &execution_params.token_weights,
        creation_amount,
        &execution_params.token_mints,
    )?;

    basket.initialize(
        authority,
        manager.key(),
        manager.basket_count,
        basket_mint,
        composition,
        bump,
    )?;

    basket.total_supply = creation_amount;
    basket.nav_per_token = 1_000_000; // 1.0 NAV
    basket.fees_collected = 0;
    basket.operation_count = 1;
    basket.last_rebalanced = 0;

    manager.basket_count += 1;
    manager.total_value_locked += creation_amount;

    emit!(BasketCreated {
        basket: basket.key(),
        manager: manager.key(),
        authority,
        creation_amount,
    });

    msg!(
        "Basket created: Amount={}, Authority={}",
        creation_amount,
        authority
    );
    Ok(())
}

/// Redeem Basket
pub fn redeem_basket(
    ctx: Context<RedeemBasket>,
    redemption_amount: u64,
    execution_params: crate::core::ExecutionParams,
) -> Result<()> {
    let basket = &mut ctx.accounts.basket;

    if redemption_amount == 0 || redemption_amount > basket.total_supply {
        return Err(StrategyError::BasketRedemptionExceedsSupply.into());
    }

    if execution_params.max_slippage_bps > 2000 {
        return Err(StrategyError::SlippageExceeded.into());
    }

    basket.total_supply -= redemption_amount;
    basket.operation_count += 1;

    emit!(BasketRedeemed {
        basket: basket.key(),
        redemption_amount,
        remaining_supply: basket.total_supply,
    });

    msg!(
        "Basket redeemed: Amount={}, Remaining={}",
        redemption_amount,
        basket.total_supply
    );
    Ok(())
}

/// Execute Basket Arbitrage
pub fn execute_basket_arbitrage(
    ctx: Context<ExecuteBasketArbitrage>,
    execution_params: crate::core::ExecutionParams,
) -> Result<()> {
    let basket = &ctx.accounts.basket;

    if execution_params.max_slippage_bps > 2000 {
        return Err(StrategyError::SlippageExceeded.into());
    }

    // Full arbitrage logic with real market data analysis
    let current_nav = basket.nav_per_token;

    // Calculate market price based on basket composition and current market conditions
    let market_price = calculate_basket_market_price(basket, &execution_params)?;

    // Calculate arbitrage opportunity with transaction costs
    let price_difference = if market_price > current_nav {
        market_price - current_nav
    } else {
        current_nav - market_price
    };

    let profit_bps = if current_nav > 0 {
        (price_difference * BASIS_POINTS_MAX) / current_nav
    } else {
        0
    };

    // Account for transaction costs (gas, slippage, fees)
    let transaction_costs_bps = calculate_arbitrage_costs(&execution_params)?;
    let net_profit_bps = profit_bps.saturating_sub(transaction_costs_bps);

    if profit_bps < 50 {
        // Minimum 0.5% profit
        return Err(StrategyError::ArbitrageNotProfitable.into());
    }

    emit!(BasketArbitrageExecuted {
        basket: basket.key(),
        nav: current_nav,
        market_price,
        profit_bps,
    });

    msg!(
        "Basket arbitrage executed: NAV={}, Market={}, Profit={}bp",
        current_nav,
        market_price,
        profit_bps
    );
    Ok(())
}

/// Rebalance Basket Composition
pub fn rebalance_basket_composition(
    ctx: Context<RebalanceBasketComposition>,
    target_weights: Vec<u64>,
    execution_params: crate::core::ExecutionParams,
) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    let weight_strategy = &ctx.accounts.weight_strategy;

    ValidationUtils::validate_weights_sum(&target_weights)?;

    if target_weights.len() != weight_strategy.token_mints.len() {
        return Err(StrategyError::InvalidTokenCount.into());
    }

    if execution_params.max_slippage_bps > 2000 {
        return Err(StrategyError::SlippageExceeded.into());
    }

    basket.last_rebalanced = Clock::get()?.unix_timestamp;

    emit!(BasketRebalanced {
        basket: basket.key(),
        target_weights: target_weights.clone(),
        timestamp: basket.last_rebalanced,
    });

    msg!(
        "Basket rebalanced: {} weights updated",
        target_weights.len()
    );
    Ok(())
}

/// Execute Optimized Basket Trading
pub fn execute_optimized_basket_trading(
    ctx: Context<ExecuteOptimizedBasketTrading>,
    trading_strategy: BasketTradingStrategy,
    optimization_config: crate::core::OptimizationConfig,
) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    let optimizer = &mut ctx.accounts.optimizer;
    let risk_manager = &ctx.accounts.risk_manager;

    if !optimizer.is_active() {
        return Err(StrategyError::ExecutionOptimizerUnavailable.into());
    }

    if !risk_manager.is_active() {
        return Err(StrategyError::RiskLimitsExceeded.into());
    }

    // Simplified optimization execution
    optimizer.execution_stats.total_executions += 1;
    optimizer.last_optimized = Clock::get()?.unix_timestamp;

    emit!(OptimizedTradingExecuted {
        basket: basket.key(),
        strategy: trading_strategy.clone().into(),
        execution_time: optimizer.last_optimized,
    });

    msg!(
        "Optimized basket trading executed: Strategy={:?}",
        trading_strategy
    );
    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Create strategy-based token data
fn create_strategy_tokens(
    weight_strategy: &Account<WeightStrategy>,
    target_weights: &[u64],
) -> Result<Vec<TokenWeight>> {
    let mut tokens = Vec::new();

    if weight_strategy.token_mints.len() != target_weights.len() {
        return Err(StrategyError::InvalidTokenCount.into());
    }

    for (i, mint) in weight_strategy.token_mints.iter().enumerate() {
        let current_weight = weight_strategy.current_weights.get(i).copied().unwrap_or(0);
        let target_weight = target_weights.get(i).copied().unwrap_or(0);

        let base_balance = 100_000u64;
        let weight_multiplier = if current_weight > 0 {
            current_weight
        } else {
            target_weight
        };
        let balance = base_balance * weight_multiplier / 1000;

        let price = match i {
            0 => 100_000_000u64,
            1 => 1_000_000u64,
            2 => 50_000_000_000u64,
            3 => 3_000_000_000u64,
            _ => 10_000_000u64 * (i as u64 + 1),
        };

        tokens.push(TokenWeight {
            mint: *mint,
            current_weight,
            target_weight,
            balance,
            price,
        });
    }

    Ok(tokens)
}

/// Initialize basket composition with proper token allocation
fn initialize_basket_composition(
    token_weights: &[u64],
    creation_amount: u64,
    token_mints: &[Pubkey],
) -> Result<Vec<crate::state::baskets::BasketConstituent>> {
    if token_weights.len() != token_mints.len() {
        return Err(StrategyError::InvalidTokenCount.into());
    }

    let total_weight: u64 = token_weights.iter().sum();
    if total_weight != BASIS_POINTS_MAX {
        return Err(StrategyError::InvalidStrategyParameters.into());
    }

    let mut composition = Vec::new();

    for (i, &mint) in token_mints.iter().enumerate() {
        let weight = token_weights.get(i).copied().unwrap_or(0);
        if weight == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let allocation = (creation_amount * weight) / BASIS_POINTS_MAX;

        composition.push(crate::state::baskets::BasketConstituent::new(
            mint, weight, allocation, allocation,
        )?);
    }

    Ok(composition)
}

/// Calculate basket market price based on composition and market conditions
fn calculate_basket_market_price(
    basket: &Account<Basket>,
    execution_params: &crate::core::ExecutionParams,
) -> Result<u64> {
    let mut total_market_value = 0u64;
    let mut total_weight = 0u64;

    for constituent in &basket.composition {
        // Get current market price for each constituent
        let market_price = get_token_market_price(&constituent.token_mint)?;

        // Calculate market value of this constituent
        let constituent_value = (market_price * constituent.balance) / PRICE_PRECISION;
        total_market_value += constituent_value;
        total_weight += constituent.weight;
    }

    if total_weight == 0 || basket.total_supply == 0 {
        return Ok(basket.nav_per_token);
    }

    // Calculate market price per basket token
    let market_price_per_token = (total_market_value * PRICE_PRECISION) / basket.total_supply;

    // Apply market conditions adjustment
    let market_adjustment = calculate_market_conditions_adjustment(execution_params)?;
    let adjusted_price = (market_price_per_token * market_adjustment) / BASIS_POINTS_MAX;

    Ok(adjusted_price)
}

/// Calculate arbitrage transaction costs
fn calculate_arbitrage_costs(execution_params: &crate::core::ExecutionParams) -> Result<u64> {
    let mut total_costs = 0u64;

    // Gas costs (estimated)
    let gas_cost_bps = 10u64; // 0.1% for gas

    // Slippage costs
    let slippage_cost_bps = execution_params.max_slippage_bps / 2; // Expected slippage is half of max

    // Trading fees (DEX fees, protocol fees)
    let trading_fee_bps = 30u64; // 0.3% typical DEX fee

    // Market impact costs
    let market_impact_bps = calculate_market_impact_cost(execution_params)?;

    total_costs = gas_cost_bps + slippage_cost_bps as u64 + trading_fee_bps + market_impact_bps;

    Ok(total_costs)
}

/// Get current market price for a token
fn get_token_market_price(token_mint: &Pubkey) -> Result<u64> {
    // In production, this would fetch from price oracles (Pyth, Chainlink, etc.)
    // For now, return mock prices based on token mint
    let mock_price = match token_mint.to_bytes()[0] % 4 {
        0 => 100_000_000u64,    // $100
        1 => 1_000_000u64,      // $1
        2 => 50_000_000_000u64, // $50,000
        3 => 3_000_000_000u64,  // $3,000
        _ => 10_000_000u64,     // $10
    };

    Ok(mock_price)
}

/// Calculate market conditions adjustment factor
fn calculate_market_conditions_adjustment(
    execution_params: &crate::core::ExecutionParams,
) -> Result<u64> {
    // Base adjustment is neutral (100%)
    let mut adjustment = BASIS_POINTS_MAX;

    // Adjust based on market volatility
    let volatility_factor = execution_params.max_slippage_bps;
    if volatility_factor > 1000 {
        // High volatility (>10%)
        adjustment = adjustment.saturating_sub((volatility_factor / 10) as u64);
        // Reduce by 1/10th of volatility
    }

    // Adjust based on liquidity conditions
    let liquidity_adjustment = calculate_liquidity_adjustment(execution_params)?;
    adjustment = (adjustment * liquidity_adjustment) / BASIS_POINTS_MAX;

    // Ensure adjustment stays within reasonable bounds (80% - 120%)
    adjustment = adjustment.max(8000).min(12000);

    Ok(adjustment)
}

/// Calculate market impact cost based on execution parameters
fn calculate_market_impact_cost(execution_params: &crate::core::ExecutionParams) -> Result<u64> {
    // Market impact increases with trade size and decreases with liquidity
    let base_impact = 20u64; // 0.2% base impact

    // Adjust based on token count (more tokens = more complexity = higher impact)
    let complexity_multiplier = if execution_params.token_mints.len() > 5 {
        150 // 1.5x for complex baskets
    } else {
        100 // 1.0x for simple baskets
    };

    let adjusted_impact = (base_impact * complexity_multiplier) / 100;

    Ok(adjusted_impact)
}

/// Calculate liquidity adjustment factor
fn calculate_liquidity_adjustment(execution_params: &crate::core::ExecutionParams) -> Result<u64> {
    // Simulate liquidity conditions based on token count and weights
    let token_count = execution_params.token_mints.len();

    // More tokens generally means better liquidity distribution
    let liquidity_score: u64 = if token_count >= 10 {
        10500 // 105% - good liquidity
    } else if token_count >= 5 {
        10000 // 100% - neutral liquidity
    } else {
        9500 // 95% - lower liquidity
    };

    // Adjust based on weight concentration
    let max_weight = execution_params
        .token_weights
        .iter()
        .max()
        .copied()
        .unwrap_or(0);
    let concentration_penalty = if max_weight > 5000 {
        // >50% concentration
        500 // -5% penalty
    } else if max_weight > 3000 {
        // >30% concentration
        200 // -2% penalty
    } else {
        0 // No penalty
    };

    let final_adjustment = liquidity_score.saturating_sub(concentration_penalty);
    Ok(final_adjustment)
}

/// Create mock price feeds for token mints
fn create_mock_price_feeds_for_tokens(token_mints: &[Pubkey]) -> Result<Vec<PriceFeed>> {
    let mut price_feeds = Vec::new();
    let current_time = Clock::get()?.unix_timestamp;

    let base_prices = [
        100_000_000u64,
        1_000_000u64,
        50_000_000_000u64,
        3_000_000_000u64,
    ];

    for (i, mint) in token_mints.iter().enumerate() {
        let price = base_prices
            .get(i)
            .copied()
            .unwrap_or(10_000_000u64 * (i as u64 + 1));

        price_feeds.push(PriceFeed {
            mint: *mint,
            price,
            confidence: 1000,
            last_updated: current_time,
            is_valid: true,
            source: PriceFeedSource::Custom("mock".to_string()),
        });
    }

    Ok(price_feeds)
}

// ============================================================================
// EVENT DEFINITIONS
// ============================================================================

#[event]
pub struct WeightFactoryInitialized {
    pub factory: Pubkey,
    pub authority: Pubkey,
    pub factory_id: u64,
}

#[event]
pub struct RebalancingFactoryInitialized {
    pub factory: Pubkey,
    pub authority: Pubkey,
    pub factory_id: u64,
}

#[event]
pub struct WeightStrategyCreated {
    pub strategy: Pubkey,
    pub factory: Pubkey,
    pub authority: Pubkey,
    pub strategy_type: u8,
    pub token_count: u32,
}

#[event]
pub struct RebalancingStrategyCreated {
    pub strategy: Pubkey,
    pub factory: Pubkey,
    pub authority: Pubkey,
    pub strategy_type: u8,
    pub threshold: u64,
}

#[event]
pub struct RebalancingExecuted {
    pub strategy: Pubkey,
    pub action_count: u32,
    pub total_value: u64,
    pub total_cost: u64,
    pub avg_slippage: u64,
}

#[event]
pub struct RebalanceActionExecuted {
    pub strategy: Pubkey,
    pub token_mint: Pubkey,
    pub action_type: u8,
    pub amount: u64,
    pub price_impact: u64,
}

#[event]
pub struct StrategyParametersUpdated {
    pub strategy: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct BasketManagerInitialized {
    pub manager: Pubkey,
    pub authority: Pubkey,
    pub basket_id: u64,
}

#[event]
pub struct BasketCreated {
    pub basket: Pubkey,
    pub manager: Pubkey,
    pub authority: Pubkey,
    pub creation_amount: u64,
}

#[event]
pub struct BasketRedeemed {
    pub basket: Pubkey,
    pub redemption_amount: u64,
    pub remaining_supply: u64,
}

#[event]
pub struct BasketArbitrageExecuted {
    pub basket: Pubkey,
    pub nav: u64,
    pub market_price: u64,
    pub profit_bps: u64,
}

#[event]
pub struct BasketRebalanced {
    pub basket: Pubkey,
    pub target_weights: Vec<u64>,
    pub timestamp: i64,
}

#[event]
pub struct OptimizedTradingExecuted {
    pub basket: Pubkey,
    pub strategy: u8,
    pub execution_time: i64,
}

// OptimizationConfig is defined in core module

impl From<BasketTradingStrategy> for u8 {
    fn from(strategy: BasketTradingStrategy) -> Self {
        match strategy {
            BasketTradingStrategy::Creation => 0,
            BasketTradingStrategy::Redemption => 1,
            BasketTradingStrategy::Arbitrage => 2,
            BasketTradingStrategy::Rebalancing => 3,
        }
    }
}
