/*!
 * Account Structures Module for Solana AMM Index Token Strategies
 *
 * This module defines all account structures used in program instructions.
 * Each structure specifies the accounts required for specific operations
 * with proper validation, security constraints, and PDA derivation.
 *
 * ## Refactored for Anchor 0.32+ Compatibility
 *
 * Key improvements:
 * - Consistent constraint patterns and error handling
 * - Proper space calculations using InitSpace trait
 * - Simplified account validation logic
 * - Enhanced security with proper PDA seed validation
 * - Optimized for gas efficiency and maintainability
 */

// Removed conflicting import
use crate::error::StrategyError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

/// Account structure for initializing a weight strategy factory
/// Updated for Anchor 0.32 with enhanced validation and space calculation
#[derive(Accounts)]
pub struct InitializeWeightFactory<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + WeightStrategyFactory::INIT_SPACE,
        seeds = [b"weight_factory"],
        bump
    )]
    pub factory: Account<'info, WeightStrategyFactory>,

    pub system_program: Program<'info, System>,
}

/// Account structure for initializing a rebalancing strategy factory
/// Updated for Anchor 0.32 with enhanced validation and space calculation
#[derive(Accounts)]
pub struct InitializeRebalancingFactory<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + RebalancingStrategyFactory::INIT_SPACE,
        seeds = [b"rebalancing_factory"],
        bump
    )]
    pub factory: Account<'info, RebalancingStrategyFactory>,

    pub system_program: Program<'info, System>,
}

/// Account structure for creating a weight strategy
#[derive(Accounts)]
pub struct CreateWeightStrategy<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = factory.base.is_active @ StrategyError::StrategyPaused
    )]
    pub factory: Account<'info, WeightStrategyFactory>,

    #[account(
        init,
        payer = authority,
        space = 8 + WeightStrategy::INIT_SPACE,
        seeds = [b"weight_strategy", factory.key().as_ref(), factory.strategy_count.to_le_bytes().as_ref()],
        bump
    )]
    pub strategy: Account<'info, WeightStrategy>,

    pub system_program: Program<'info, System>,
}

/// Account structure for creating a rebalancing strategy
#[derive(Accounts)]
pub struct CreateRebalancingStrategy<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = factory.base.is_active @ StrategyError::StrategyPaused
    )]
    pub factory: Account<'info, RebalancingStrategyFactory>,

    #[account(
        init,
        payer = authority,
        space = 8 + crate::state::strategies::RebalancingStrategy::INIT_SPACE,
        seeds = [b"rebalancing_strategy", factory.key().as_ref(), factory.strategy_count.to_le_bytes().as_ref()],
        bump
    )]
    pub strategy: Account<'info, crate::state::strategies::RebalancingStrategy>,

    pub system_program: Program<'info, System>,
}

/// Account structure for executing rebalancing
#[derive(Accounts)]
pub struct ExecuteRebalancing<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = rebalancing_strategy.base.is_active @ StrategyError::StrategyPaused,
        constraint = !rebalancing_strategy.base.is_paused @ StrategyError::StrategyPaused
    )]
    pub rebalancing_strategy: Account<'info, crate::state::strategies::RebalancingStrategy>,

    #[account(
        constraint = weight_strategy.key() == rebalancing_strategy.weight_strategy @ StrategyError::InvalidStrategyParameters
    )]
    pub weight_strategy: Account<'info, WeightStrategy>,
}

/// Account structure for updating strategy parameters
#[derive(Accounts)]
pub struct UpdateStrategyParams<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = strategy.base.is_active @ StrategyError::StrategyPaused
    )]
    pub strategy: Account<'info, WeightStrategy>,
}

/// Account structure for initializing basket manager
#[derive(Accounts)]
pub struct InitializeBasketManager<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + BasketManager::INIT_SPACE,
        seeds = [b"basket_manager"],
        bump
    )]
    pub manager: Account<'info, BasketManager>,

    pub system_program: Program<'info, System>,
}

/// Account structure for creating a basket
#[derive(Accounts)]
pub struct CreateBasket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = manager.base.is_active @ StrategyError::StrategyPaused
    )]
    pub manager: Account<'info, BasketManager>,

    #[account(
        init,
        payer = authority,
        space = 8 + BasketInstance::INIT_SPACE,
        seeds = [b"basket", manager.key().as_ref(), manager.basket_count.to_le_bytes().as_ref()],
        bump
    )]
    pub basket: Account<'info, BasketInstance>,

    pub system_program: Program<'info, System>,
}

/// Account structure for redeeming a basket
#[derive(Accounts)]
pub struct RedeemBasket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = basket.base.is_active @ StrategyError::StrategyPaused,
        constraint = !basket.base.is_paused @ StrategyError::StrategyPaused
    )]
    pub basket: Account<'info, BasketInstance>,
}

/// Account structure for basket arbitrage
#[derive(Accounts)]
pub struct ExecuteBasketArbitrage<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = basket.base.is_active @ StrategyError::StrategyPaused
    )]
    pub basket: Account<'info, BasketInstance>,
}

/// Account structure for rebalancing basket composition
#[derive(Accounts)]
pub struct RebalanceBasketComposition<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = basket.base.is_active @ StrategyError::StrategyPaused
    )]
    pub basket: Account<'info, BasketInstance>,

    pub weight_strategy: Account<'info, WeightStrategy>,
}

/// Account structure for optimized basket trading
#[derive(Accounts)]
pub struct ExecuteOptimizedBasketTrading<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = basket.base.is_active @ StrategyError::StrategyPaused
    )]
    pub basket: Account<'info, BasketInstance>,

    #[account(
        mut,
        constraint = optimizer.base.is_active @ StrategyError::ExecutionOptimizerUnavailable
    )]
    pub optimizer: Account<'info, ExecutionOptimizer>,

    #[account(
        mut,
        constraint = risk_manager.base.is_active @ StrategyError::RiskLimitsExceeded
    )]
    pub risk_manager: Account<'info, RiskManager>,
}

/// Account structure for initializing index token manager
#[derive(Accounts)]
pub struct InitializeIndexTokenManager<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + crate::index_tokens::IndexTokenManager::INIT_SPACE,
        seeds = [b"index_token_manager"],
        bump
    )]
    pub manager: Account<'info, crate::index_tokens::IndexTokenManager>,

    pub fee_collector: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

/// Account structure for creating index token
#[derive(Accounts)]
pub struct CreateIndexToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = manager.base.is_active @ StrategyError::StrategyPaused
    )]
    pub manager: Account<'info, crate::index_tokens::IndexTokenManager>,

    #[account(
        init,
        payer = authority,
        space = 8 + crate::index_tokens::IndexToken::INIT_SPACE,
        seeds = [b"index_token", manager.key().as_ref(), manager.token_count.to_le_bytes().as_ref()],
        bump
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    pub token_mint: SystemAccount<'info>,
    pub weight_strategy: Account<'info, WeightStrategy>,
    pub system_program: Program<'info, System>,
}

/// Account structure for creating index token units
#[derive(Accounts)]
pub struct CreateIndexTokenUnits<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    #[account(mut)]
    pub manager: Account<'info, crate::index_tokens::IndexTokenManager>,

    /// CHECK: Mint validation handled by token program
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,

    /// CHECK: Token account validation handled by token program
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,

    /// CHECK: Mint authority validation handled by token program
    pub mint_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Account structure for redeeming index token units
#[derive(Accounts)]
pub struct RedeemIndexTokenUnits<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    #[account(mut)]
    pub manager: Account<'info, crate::index_tokens::IndexTokenManager>,

    /// CHECK: Mint validation handled by token program
    #[account(mut)]
    pub token_mint: AccountInfo<'info>,

    /// CHECK: Token account validation handled by token program
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

/// Account structure for rebalancing index token
#[derive(Accounts)]
pub struct RebalanceIndexToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    pub weight_strategy: Account<'info, WeightStrategy>,
}

/// Account structure for index token arbitrage
#[derive(Accounts)]
pub struct ExecuteIndexTokenArbitrage<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,
}

/// Account structure for calculating index token NAV
#[derive(Accounts)]
pub struct CalculateIndexTokenNav<'info> {
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,
}

/// Account structure for advanced index token trading
#[derive(Accounts)]
pub struct ExecuteAdvancedIndexTokenTrading<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    #[account(
        mut,
        constraint = trading_engine.base.is_active @ StrategyError::ExecutionOptimizerUnavailable
    )]
    pub trading_engine: Account<'info, ExecutionOptimizer>,

    #[account(
        mut,
        constraint = risk_manager.base.is_active @ StrategyError::RiskLimitsExceeded
    )]
    pub risk_manager: Account<'info, RiskManager>,
}

/// Account structure for index token market making
#[derive(Accounts)]
pub struct ExecuteIndexTokenMarketMaking<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    #[account(
        mut,
        constraint = market_maker_state.base.is_active @ StrategyError::ExecutionOptimizerUnavailable
    )]
    pub market_maker_state: Account<'info, ExecutionOptimizer>,
}

/// Account structure for cross-AMM arbitrage
#[derive(Accounts)]
pub struct ExecuteCrossAMMArbitrage<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    #[account(
        mut,
        constraint = arbitrage_engine.base.is_active @ StrategyError::ExecutionOptimizerUnavailable
    )]
    pub arbitrage_engine: Account<'info, ExecutionOptimizer>,
}

/// Account structure for liquidity provision
#[derive(Accounts)]
pub struct ExecuteIndexTokenLiquidityProvision<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    #[account(
        mut,
        constraint = liquidity_manager.base.is_active @ StrategyError::ExecutionOptimizerUnavailable
    )]
    pub liquidity_manager: Account<'info, ExecutionOptimizer>,
}

/// Account structure for algorithmic trading
#[derive(Accounts)]
pub struct ExecuteAlgorithmicIndexTokenTrading<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    #[account(
        mut,
        constraint = algo_trading_engine.base.is_active @ StrategyError::ExecutionOptimizerUnavailable
    )]
    pub algo_trading_engine: Account<'info, ExecutionOptimizer>,

    #[account(
        mut,
        constraint = signal_processor.base.is_active @ StrategyError::ExecutionOptimizerUnavailable
    )]
    pub signal_processor: Account<'info, ExecutionOptimizer>,
}

/// Account structure for portfolio optimization
#[derive(Accounts)]
pub struct ExecuteIndexTokenPortfolioOptimization<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = base.authority @ StrategyError::Unauthorized,
        constraint = index_token.base.is_active @ StrategyError::StrategyPaused
    )]
    pub index_token: Account<'info, crate::index_tokens::IndexToken>,

    #[account(
        mut,
        constraint = portfolio_optimizer.base.is_active @ StrategyError::ExecutionOptimizerUnavailable
    )]
    pub portfolio_optimizer: Account<'info, ExecutionOptimizer>,

    #[account(
        mut,
        constraint = risk_model.base.is_active @ StrategyError::RiskLimitsExceeded
    )]
    pub risk_model: Account<'info, RiskManager>,

    #[account(
        constraint = weight_strategy.base.is_active @ StrategyError::StrategyPaused
    )]
    pub weight_strategy: Account<'info, WeightStrategy>,
}
