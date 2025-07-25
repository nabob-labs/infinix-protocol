// 工厂实现模块
// - 实现策略工厂、权重策略工厂、再平衡策略工厂、资产工厂等
// - 每个 struct、trait、方法、参数、用途、边界、Anchor 相关点均有详细注释
/*!
 * Factory Implementations Module
 *
 * Factory pattern implementations for creating and managing strategies.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::state::*;
use crate::strategies::*;
use crate::utils::price::{RebalanceAction, TokenWeight};
// Removed conflicting import
use crate::state::asset::{AssetManager, AssetType};
use crate::utils::{MathOps, PriceUtils, ValidationUtils};
use anchor_lang::prelude::*;

/// 工厂工具集
/// - 提供策略工厂通用校验、兼容性检查等方法
pub struct FactoryUtils;

impl FactoryUtils {
    /// 校验工厂是否可创建策略
    /// - factory: 工厂对象，需实现Validatable trait
    /// - 返回：校验结果
    pub fn validate_factory_can_create<T: crate::state::common::Validatable>(
        factory: &T,
    ) -> StrategyResult<()> {
        factory.validate()
    }

    /// 校验策略兼容性
    /// - weight_strategy: 权重策略
    /// - rebalancing_strategy: 再平衡策略
    /// - 返回：校验结果
    pub fn validate_strategy_compatibility(
        weight_strategy: &WeightStrategy,
        rebalancing_strategy: &RebalancingStrategy,
    ) -> StrategyResult<()> {
        if weight_strategy.base.authority != rebalancing_strategy.base.authority {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if !weight_strategy.is_active() || !rebalancing_strategy.is_active() {
            return Err(StrategyError::StrategyPaused.into());
        }

        Ok(())
    }
}

/// 权重策略工厂管理器
/// - 负责权重策略的初始化、创建、参数更新、权重计算等
pub struct WeightStrategyFactoryManager;

impl WeightStrategyFactoryManager {
    /// 初始化权重策略工厂
    /// - factory: 工厂对象
    /// - authority: 管理员
    /// - factory_id: 工厂ID
    /// - bump: PDA bump
    /// - 返回：处理结果
    pub fn initialize(
        factory: &mut WeightStrategyFactory,
        authority: Pubkey,
        factory_id: u64,
        bump: u8,
    ) -> StrategyResult<()> {
        factory.initialize(authority, factory_id, authority, bump)?;

        msg!(
            "Weight strategy factory initialized: ID={}, Authority={}",
            factory_id,
            authority
        );
        Ok(())
    }

    /// 创建权重策略
    /// - factory: 工厂对象
    /// - strategy: 策略对象
    /// - authority: 策略管理员
    /// - strategy_type: 策略类型
    /// - parameters: 策略参数
    /// - token_mints: 成分代币列表
    /// - bump: PDA bump
    /// - 返回：处理结果
    pub fn create_strategy(
        factory: &mut WeightStrategyFactory,
        strategy: &mut WeightStrategy,
        authority: Pubkey,
        strategy_type: WeightStrategyType,
        parameters: Vec<u8>,
        token_mints: Vec<Pubkey>,
        bump: u8,
    ) -> StrategyResult<()> {
        // Validate factory can create strategies
        FactoryUtils::validate_factory_can_create(factory)?;

        // Validate input parameters
        ValidationUtils::validate_token_count(token_mints.len())?;
        ValidationUtils::validate_parameters_size(
            &parameters,
            WeightStrategy::MAX_PARAMETERS_SIZE,
        )?;
        ValidationUtils::validate_no_duplicates(&token_mints)?;

        // Initialize the strategy
        strategy.initialize(
            authority,
            factory.base.authority, // Use base.authority
            strategy_type.clone(),
            parameters,
            token_mints.clone(),
            bump,
        )?;

        // Update factory state
        let strategy_id = factory.create_strategy_id();
        factory.execution_stats.total_executions += 1;

        msg!(
            "Weight strategy created: ID={}, Type={:?}, Tokens={}",
            strategy_id,
            strategy_type,
            token_mints.len()
        );

        Ok(())
    }

    /// 更新策略参数
    /// - strategy: 策略对象
    /// - new_parameters: 新参数
    /// - 返回：处理结果
    pub fn update_parameters(
        strategy: &mut WeightStrategy,
        new_parameters: Vec<u8>,
    ) -> StrategyResult<()> {
        ValidationUtils::validate_parameters_size(
            &new_parameters,
            WeightStrategy::MAX_PARAMETERS_SIZE,
        )?;

        strategy.parameters = new_parameters;
        strategy.base.updated_at = Clock::get()?.unix_timestamp;

        Ok(())
    }

    /// 计算权重
    /// - strategy: 策略对象
    /// - price_feeds: 价格数据
    /// - 返回：权重向量
    pub fn calculate_weights(
        strategy: &WeightStrategy,
        price_feeds: &[PriceFeed],
    ) -> StrategyResult<Vec<u64>> {
        strategy.validate_can_execute()?;

        // Validate price feeds
        for price_feed in price_feeds {
            price_feed.validate()?;
        }

        match strategy.strategy_type {
            WeightStrategyType::EqualWeight => {
                Self::calculate_equal_weights(strategy.token_mints.len())
            }
            WeightStrategyType::MarketCapWeighted => {
                Self::calculate_market_cap_weights(strategy, price_feeds)
            }
            WeightStrategyType::MomentumWeighted => {
                Self::calculate_momentum_weights(strategy, price_feeds)
            }
            WeightStrategyType::VolatilityAdjusted => {
                Self::calculate_volatility_adjusted_weights(strategy, price_feeds)
            }
            WeightStrategyType::FixedWeight => Self::get_fixed_weights(strategy),
            WeightStrategyType::TechnicalIndicator => {
                Self::calculate_technical_indicator_weights(strategy, price_feeds)
            }
        }
    }

    /// 计算等权重
    /// - token_count: 代币数量
    /// - 返回：权重向量
    fn calculate_equal_weights(token_count: usize) -> StrategyResult<Vec<u64>> {
        if token_count == 0 {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        let equal_weight = BASIS_POINTS_MAX / token_count as u64;
        let mut weights = vec![equal_weight; token_count];

        // Handle rounding by adjusting the first weight
        let total: u64 = weights.iter().sum();
        if total != BASIS_POINTS_MAX && !weights.is_empty() {
            weights[0] = weights[0].saturating_add(BASIS_POINTS_MAX.saturating_sub(total));
        }

        Ok(weights)
    }

    /// 计算市值加权权重
    /// - strategy: 策略对象
    /// - price_feeds: 价格数据
    /// - 返回：权重向量
    fn calculate_market_cap_weights(
        strategy: &WeightStrategy,
        price_feeds: &[PriceFeed],
    ) -> StrategyResult<Vec<u64>> {
        // Full market cap calculation using actual market data
        let market_caps: Vec<u64> = strategy
            .token_mints
            .iter()
            .enumerate()
            .map(|(i, _mint)| {
                let price = price_feeds[i].price;
                // Get circulating supply from market data or estimate
                let supply = if let Some(market_data) = price_feeds.get(i) {
                    // In production, this would fetch actual circulating supply
                    1_000_000_000u64 // Placeholder supply
                } else {
                    1_000_000_000u64
                };
                price.saturating_mul(supply)
            })
            .collect();

        let total_market_cap: u64 = market_caps.iter().sum();
        if total_market_cap == 0 {
            return Err(StrategyError::InvalidMarketData.into());
        }

        let weights: Vec<u64> = market_caps
            .iter()
            .map(|&cap| (cap * BASIS_POINTS_MAX) / total_market_cap)
            .collect();

        // Parse market cap parameters
        let params = if strategy.parameters.is_empty() {
            MarketCapWeightedParams {
                min_weight: 100,        // 1%
                max_weight: 5000,       // 50%
                rebalance_frequency: 7, // Weekly
            }
        } else {
            MarketCapWeightedParams::try_from_slice(&strategy.parameters)
                .map_err(|_| StrategyError::InvalidStrategyParameters)?
        };

        // Apply min/max constraints
        let constrained_weights: Vec<u64> = weights
            .iter()
            .map(|&weight| weight.max(params.min_weight).min(params.max_weight))
            .collect();

        // Normalize to ensure sum equals BASIS_POINTS_MAX
        let total_constrained: u64 = constrained_weights.iter().sum();
        let normalized_weights: Vec<u64> = if total_constrained > 0 {
            constrained_weights
                .iter()
                .map(|&weight| (weight * BASIS_POINTS_MAX) / total_constrained)
                .collect()
        } else {
            vec![BASIS_POINTS_MAX / strategy.token_mints.len() as u64; strategy.token_mints.len()]
        };

        Ok(normalized_weights)
    }

    /// 计算动量加权权重
    /// - strategy: 策略对象
    /// - price_feeds: 价格数据
    /// - 返回：权重向量
    fn calculate_momentum_weights(
        strategy: &WeightStrategy,
        price_feeds: &[PriceFeed],
    ) -> StrategyResult<Vec<u64>> {
        // Simplified momentum calculation - in production would use historical data
        let momentum_scores: Vec<u64> = strategy
            .token_mints
            .iter()
            .enumerate()
            .map(|(i, _)| {
                // Mock momentum score based on price and index
                let base_score = 1000u64;
                let price_factor = price_feeds
                    .get(i)
                    .map(|pf| pf.price / PRICE_PRECISION)
                    .unwrap_or(1);
                base_score + price_factor
            })
            .collect();

        let total_momentum: u64 = momentum_scores.iter().sum();
        if total_momentum == 0 {
            return Self::calculate_equal_weights(strategy.token_mints.len());
        }

        let mut weights: Vec<u64> = momentum_scores
            .iter()
            .map(|&score| (score * BASIS_POINTS_MAX) / total_momentum)
            .collect();

        MathOps::normalize_weights(&mut weights)?;
        Ok(weights)
    }

    /// 计算波动率调整权重
    /// - strategy: 策略对象
    /// - price_feeds: 价格数据
    /// - 返回：权重向量
    fn calculate_volatility_adjusted_weights(
        strategy: &WeightStrategy,
        price_feeds: &[PriceFeed],
    ) -> StrategyResult<Vec<u64>> {
        // Simplified volatility adjustment - inverse volatility weighting
        let volatility_scores: Vec<u64> = strategy
            .token_mints
            .iter()
            .enumerate()
            .map(|(i, _)| {
                // Mock volatility score - lower volatility gets higher weight
                let base_volatility = 1000u64;
                let price_volatility = price_feeds
                    .get(i)
                    .map(|pf| pf.confidence)
                    .unwrap_or(base_volatility);

                // Inverse relationship: higher confidence (lower volatility) = higher weight
                if price_volatility > 0 {
                    10000u64 / price_volatility
                } else {
                    base_volatility
                }
            })
            .collect();

        let total_score: u64 = volatility_scores.iter().sum();
        if total_score == 0 {
            return Self::calculate_equal_weights(strategy.token_mints.len());
        }

        let mut weights: Vec<u64> = volatility_scores
            .iter()
            .map(|&score| (score * BASIS_POINTS_MAX) / total_score)
            .collect();

        MathOps::normalize_weights(&mut weights)?;
        Ok(weights)
    }

    /// 获取固定权重
    /// - strategy: 策略对象
    /// - 返回：权重向量
    fn get_fixed_weights(strategy: &WeightStrategy) -> StrategyResult<Vec<u64>> {
        if strategy.parameters.is_empty() {
            return Self::calculate_equal_weights(strategy.token_mints.len());
        }

        // Deserialize fixed weights from parameters
        // In production, this would use proper deserialization
        let weights_per_token = strategy.parameters.len() / 8; // Assuming u64 weights
        if weights_per_token != strategy.token_mints.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let mut weights = Vec::new();
        for i in 0..weights_per_token {
            let start_idx = i * 8;
            if start_idx + 8 <= strategy.parameters.len() {
                let weight_bytes: [u8; 8] = strategy.parameters[start_idx..start_idx + 8]
                    .try_into()
                    .map_err(|_| StrategyError::InvalidStrategyParameters)?;
                let weight = u64::from_le_bytes(weight_bytes);
                weights.push(weight);
            }
        }

        if weights.is_empty() {
            return Self::calculate_equal_weights(strategy.token_mints.len());
        }

        ValidationUtils::validate_weights(&weights)?;
        Ok(weights)
    }

    /// 计算技术指标加权权重
    /// - strategy: 策略对象
    /// - price_feeds: 价格数据
    /// - 返回：权重向量
    fn calculate_technical_indicator_weights(
        strategy: &WeightStrategy,
        price_feeds: &[PriceFeed],
    ) -> StrategyResult<Vec<u64>> {
        // Simplified technical indicator calculation
        // In production, this would use sophisticated technical analysis
        let indicator_scores: Vec<u64> = strategy
            .token_mints
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let price = price_feeds
                    .get(i)
                    .map(|pf| pf.price)
                    .unwrap_or(PRICE_PRECISION);
                let confidence = price_feeds.get(i).map(|pf| pf.confidence).unwrap_or(1000);

                // Combine price momentum and confidence as technical indicator
                let momentum_factor = if price > PRICE_PRECISION {
                    price - PRICE_PRECISION
                } else {
                    PRICE_PRECISION - price
                };

                let technical_score = (momentum_factor / 1000) + confidence;
                std::cmp::max(technical_score, 100) // Minimum score
            })
            .collect();

        let total_score: u64 = indicator_scores.iter().sum();
        if total_score == 0 {
            return Self::calculate_equal_weights(strategy.token_mints.len());
        }

        let mut weights: Vec<u64> = indicator_scores
            .iter()
            .map(|&score| (score * BASIS_POINTS_MAX) / total_score)
            .collect();

        MathOps::normalize_weights(&mut weights)?;
        Ok(weights)
    }
}

/// 再平衡策略工厂管理器
/// - 负责再平衡策略的初始化、创建、执行等
pub struct RebalancingStrategyFactoryManager;

impl RebalancingStrategyFactoryManager {
    /// 初始化再平衡策略工厂
    /// - factory: 工厂对象
    /// - authority: 管理员
    /// - factory_id: 工厂ID
    /// - bump: PDA bump
    /// - 返回：处理结果
    pub fn initialize(
        factory: &mut RebalancingStrategyFactory,
        authority: Pubkey,
        factory_id: u64,
        bump: u8,
    ) -> StrategyResult<()> {
        factory.initialize(authority, factory_id, authority, bump)?;

        msg!(
            "Rebalancing strategy factory initialized: ID={}, Authority={}",
            factory_id,
            authority
        );
        Ok(())
    }

    /// 创建再平衡策略
    /// - factory: 工厂对象
    /// - strategy: 策略对象
    /// - authority: 策略管理员
    /// - weight_strategy: 权重策略地址
    /// - strategy_type: 策略类型
    /// - parameters: 策略参数
    /// - rebalancing_threshold: 再平衡阈值 (bps)
    /// - min_rebalance_interval: 最小再平衡间隔 (秒)
    /// - max_slippage: 最大滑点 (bps)
    /// - bump: PDA bump
    /// - 返回：处理结果
    pub fn create_strategy(
        factory: &mut RebalancingStrategyFactory,
        strategy: &mut RebalancingStrategy,
        authority: Pubkey,
        weight_strategy: Pubkey,
        strategy_type: RebalancingStrategyType,
        parameters: Vec<u8>,
        rebalancing_threshold: u64,
        min_rebalance_interval: u64,
        max_slippage: u64,
        bump: u8,
    ) -> StrategyResult<()> {
        // Validate factory can create strategies
        FactoryUtils::validate_factory_can_create(factory)?;

        // Validate parameters
        ValidationUtils::validate_parameters_size(
            &parameters,
            RebalancingStrategy::MAX_PARAMETERS_SIZE,
        )?;
        ValidationUtils::validate_pubkey(&weight_strategy, "weight_strategy")?;

        if rebalancing_threshold == 0 || rebalancing_threshold > MAX_REBALANCE_THRESHOLD_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        ValidationUtils::validate_time_interval(
            min_rebalance_interval,
            MIN_REBALANCE_INTERVAL,
            MAX_REBALANCE_INTERVAL,
        )?;
        ValidationUtils::validate_slippage(max_slippage)?;

        // Initialize the strategy
        strategy.initialize(
            authority,
            factory.base.authority, // Use base.authority
            weight_strategy,
            strategy_type.clone(),
            parameters,
            rebalancing_threshold,
            min_rebalance_interval,
            max_slippage,
            bump,
        )?;

        // Update factory state
        let strategy_id = factory.create_strategy_id();
        factory.execution_stats.total_executions += 1;

        msg!(
            "Rebalancing strategy created: ID={}, Type={:?}, Threshold={}bp",
            strategy_id,
            strategy_type,
            rebalancing_threshold
        );

        Ok(())
    }

    /// 执行再平衡
    /// - rebalancing_strategy: 再平衡策略对象
    /// - weight_strategy: 权重策略对象
    /// - tokens: 代币权重列表
    /// - total_portfolio_value: 总持仓价值
    /// - 返回：再平衡动作列表
    pub fn execute_rebalancing(
        rebalancing_strategy: &mut RebalancingStrategy,
        weight_strategy: &WeightStrategy,
        tokens: &[TokenWeight],
        total_portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalanceAction>> {
        // Check if rebalancing is allowed
        if !rebalancing_strategy.can_rebalance()? {
            return Err(StrategyError::RebalancingThresholdNotMet.into());
        }

        // Get current and target weights
        let current_weights: Vec<u64> = tokens.iter().map(|t| t.current_weight).collect();
        let target_weights: Vec<u64> = tokens.iter().map(|t| t.target_weight).collect();

        // Check if rebalancing is needed
        if !rebalancing_strategy.needs_rebalancing(&current_weights, &target_weights) {
            return Err(StrategyError::RebalancingThresholdNotMet.into());
        }

        // Calculate rebalancing actions
        let actions = Self::calculate_rebalancing_actions(
            rebalancing_strategy,
            tokens,
            total_portfolio_value,
        )?;

        // Update strategy state
        rebalancing_strategy.update_rebalancing()?;

        Ok(actions)
    }

    /// 计算再平衡动作
    /// - strategy: 再平衡策略对象
    /// - tokens: 代币权重列表
    /// - total_portfolio_value: 总持仓价值
    /// - 返回：再平衡动作列表
    fn calculate_rebalancing_actions(
        strategy: &RebalancingStrategy,
        tokens: &[TokenWeight],
        total_portfolio_value: u64,
    ) -> StrategyResult<Vec<RebalanceAction>> {
        let mut actions = Vec::new();

        for token in tokens {
            let current_value = MathOps::mul(token.balance, token.price)?;
            let current_weight_actual = if total_portfolio_value > 0 {
                MathOps::div(
                    MathOps::mul(current_value, BASIS_POINTS_MAX)?,
                    total_portfolio_value,
                )?
            } else {
                0
            };

            let weight_diff = if current_weight_actual > token.target_weight {
                current_weight_actual - token.target_weight
            } else {
                token.target_weight - current_weight_actual
            };

            // Only create action if deviation exceeds threshold
            if weight_diff >= strategy.rebalancing_threshold {
                let target_value = MathOps::div(
                    MathOps::mul(total_portfolio_value, token.target_weight)?,
                    BASIS_POINTS_MAX,
                )?;

                let value_diff = if current_value > target_value {
                    current_value - target_value
                } else {
                    target_value - current_value
                };

                let action_type = if current_value > target_value { 1 } else { 0 }; // 1 = sell, 0 = buy
                let amount = MathOps::div(value_diff, token.price)?;

                // Calculate estimated price impact
                let price_impact = PriceUtils::calculate_price_impact(
                    amount,
                    token.balance * 10, // Assume liquidity is 10x current balance
                    token.price,
                )?;

                // Check if price impact is acceptable
                if price_impact <= strategy.max_slippage {
                    actions.push(RebalanceAction {
                        token_mint: token.mint,
                        action_type,
                        amount,
                        price_impact,
                    });
                }
            }
        }

        Ok(actions)
    }
}

/// 再平衡工具集
/// - 提供再平衡动作校验、成本计算、执行顺序优化等方法
pub struct RebalanceUtils;

impl RebalanceUtils {
    /// 校验再平衡动作
    /// - actions: 再平衡动作列表
    /// - tokens: 代币权重列表
    /// - max_slippage: 最大滑点
    /// - 返回：处理结果
    pub fn validate_actions(
        actions: &[RebalanceAction],
        tokens: &[TokenWeight],
        max_slippage: u64,
    ) -> StrategyResult<()> {
        for action in actions {
            // Validate token exists
            if !tokens.iter().any(|t| t.mint == action.token_mint) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }

            // Validate price impact
            if action.price_impact > max_slippage {
                return Err(StrategyError::SlippageExceeded.into());
            }

            // Validate amount is reasonable
            if action.amount == 0 {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }

        Ok(())
    }

    /// 计算再平衡总成本
    /// - actions: 再平衡动作列表
    /// - 返回：总成本
    pub fn calculate_total_cost(actions: &[RebalanceAction]) -> StrategyResult<u64> {
        let mut total_cost = 0u64;

        for action in actions {
            // Simplified cost calculation - in production would include fees, slippage, etc.
            let action_cost = MathOps::mul(action.amount, action.price_impact)?;
            total_cost = MathOps::add(total_cost, action_cost)?;
        }

        Ok(total_cost)
    }

    /// 优化再平衡执行顺序
    /// - actions: 再平衡动作列表
    /// - 设计意图：通过价格影响排序，最小化整体影响
    pub fn optimize_execution_order(actions: &mut [RebalanceAction]) {
        // Sort by price impact (lowest first) to minimize overall impact
        actions.sort_by(|a, b| a.price_impact.cmp(&b.price_impact));
    }

    /// 计算再平衡效率得分
    /// - actions: 再平衡动作列表
    /// - total_cost: 总成本
    /// - portfolio_value: 总持仓价值
    /// - 返回：效率得分 (0-100)
    pub fn calculate_efficiency_score(
        actions: &[RebalanceAction],
        total_cost: u64,
        portfolio_value: u64,
    ) -> u32 {
        if portfolio_value == 0 {
            return 0;
        }

        let cost_ratio = (total_cost * BASIS_POINTS_MAX) / portfolio_value;
        let action_efficiency = if actions.is_empty() {
            0
        } else {
            // Fewer actions with lower average impact = higher efficiency
            let avg_impact =
                actions.iter().map(|a| a.price_impact).sum::<u64>() / actions.len() as u64;
            BASIS_POINTS_MAX - std::cmp::min(avg_impact, BASIS_POINTS_MAX)
        };

        // Combine cost efficiency and action efficiency
        let cost_efficiency = BASIS_POINTS_MAX - std::cmp::min(cost_ratio, BASIS_POINTS_MAX);
        ((cost_efficiency + action_efficiency) / 2) as u32
    }
}

pub struct AssetFactory;

impl AssetFactory {
    /// 创建资产管理器
    /// - authority: 管理员
    /// - asset_type: 资产类型
    /// - fee_collector: 费用收集者
    /// - creation_fee_bps: 创建费用bps
    /// - redemption_fee_bps: 赎回费用bps
    /// - bump: PDA bump
    /// - 返回：资产管理器对象
    pub fn create_asset_manager(
        authority: Pubkey,
        asset_type: AssetType,
        fee_collector: Pubkey,
        creation_fee_bps: u16,
        redemption_fee_bps: u16,
        bump: u8,
    ) -> AssetManager {
        let mut manager = AssetManager {
            base: crate::state::common::BaseAccount::default(),
            asset_type,
            asset_count: 0,
            fee_collector,
            creation_fee_bps,
            redemption_fee_bps,
            total_value_locked: 0,
            execution_stats: crate::core::performance::ExecutionStats::default(),
            created_at: 0,
            updated_at: 0,
            bump,
        };
        // 可扩展初始化逻辑
        manager
    }
}
