/*!
 * Strategy Factory Implementation
 *
 * Factory pattern for creating and managing different strategy types.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// Strategy factory for creating different types of strategies
pub struct StrategyFactory;

impl StrategyFactory {
    /// Create a weight strategy based on type and parameters
    pub fn create_weight_strategy(
        strategy_type: WeightStrategyType,
        parameters: &[u8],
        token_mints: &[Pubkey],
    ) -> StrategyResult<WeightStrategyConfig> {
        // Validate inputs
        if token_mints.is_empty() || token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        if parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let current_time = Clock::get()?.unix_timestamp;

        let config = WeightStrategyConfig {
            strategy_type,
            parameters: parameters.to_vec(),
            token_mints: token_mints.to_vec(),
            is_active: true,
            last_calculation: 0,
        };

        // Validate the created configuration
        Self::validate_weight_strategy_config(&config)?;

        Ok(config)
    }

    /// Create a rebalancing strategy based on type and parameters
    pub fn create_rebalancing_strategy(
        strategy_type: RebalancingStrategyType,
        parameters: &[u8],
        rebalancing_threshold: u64,
        min_interval: u64,
    ) -> StrategyResult<RebalancingStrategyConfig> {
        // Validate inputs
        if parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if rebalancing_threshold == 0 || rebalancing_threshold > MAX_REBALANCE_THRESHOLD_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if min_interval < MIN_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }

        let current_time = Clock::get()?.unix_timestamp;

        let config = RebalancingStrategyConfig {
            strategy_type,
            parameters: parameters.to_vec(),
            is_active: true,
            last_rebalance: 0,
            next_rebalance: current_time + min_interval as i64,
        };

        // Validate the created configuration
        Self::validate_rebalancing_strategy_config(&config)?;

        Ok(config)
    }

    /// Create optimization settings with default values
    pub fn create_optimization_settings() -> OptimizationSettings {
        OptimizationSettings {
            enable_caching: true,
            enable_parallel: false,
            batch_size: DEFAULT_BATCH_SIZE,
            cache_expiry: DEFAULT_CACHE_TTL as u64,
        }
    }

    /// Create risk settings with default values
    pub fn create_risk_settings() -> RiskSettings {
        RiskSettings {
            max_concentration: DEFAULT_CONCENTRATION_LIMIT_BPS,
            max_daily_rebalances: 4,
            enable_circuit_breakers: true,
            volatility_circuit_breaker: CIRCUIT_BREAKER_THRESHOLD_BPS,
        }
    }

    /// Create a complete strategy configuration
    pub fn create_strategy_config(
        config_id: u64,
        authority: Pubkey,
        weight_config: WeightStrategyConfig,
        rebalancing_config: RebalancingStrategyConfig,
    ) -> StrategyResult<StrategyConfig> {
        // Validate compatibility between weight and rebalancing strategies
        Self::validate_strategy_compatibility(&weight_config, &rebalancing_config)?;

        let config = StrategyConfig::new(config_id, authority, weight_config, rebalancing_config)?;

        Ok(config)
    }

    /// Validate weight strategy configuration
    fn validate_weight_strategy_config(config: &WeightStrategyConfig) -> StrategyResult<()> {
        // Check token count
        if config.token_mints.is_empty() || config.token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        // Check for duplicate token mints
        let mut seen = std::collections::HashSet::new();
        for mint in &config.token_mints {
            if !seen.insert(*mint) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }

        // Validate strategy-specific parameters
        match config.strategy_type {
            WeightStrategyType::EqualWeight => {
                // No additional validation needed
            }
            WeightStrategyType::MarketCapWeighted => {
                Self::validate_market_cap_params(&config.parameters)?;
            }
            WeightStrategyType::MomentumWeighted => {
                Self::validate_momentum_params(&config.parameters)?;
            }
            WeightStrategyType::VolatilityAdjusted => {
                Self::validate_volatility_params(&config.parameters)?;
            }
            WeightStrategyType::FixedWeight => {
                Self::validate_fixed_weight_params(&config.parameters, config.token_mints.len())?;
            }
            WeightStrategyType::TechnicalIndicator => {
                Self::validate_technical_params(&config.parameters)?;
            }
        }

        Ok(())
    }

    /// Validate rebalancing strategy configuration
    fn validate_rebalancing_strategy_config(
        config: &RebalancingStrategyConfig,
    ) -> StrategyResult<()> {
        // Validate strategy-specific parameters
        match config.strategy_type {
            RebalancingStrategyType::ThresholdBased => {
                Self::validate_threshold_params(&config.parameters)?;
            }
            RebalancingStrategyType::TimeBased => {
                Self::validate_time_params(&config.parameters)?;
            }
            RebalancingStrategyType::VolatilityTriggered => {
                Self::validate_volatility_trigger_params(&config.parameters)?;
            }
            RebalancingStrategyType::DriftBased => {
                Self::validate_drift_params(&config.parameters)?;
            }
            RebalancingStrategyType::Hybrid => {
                Self::validate_hybrid_params(&config.parameters)?;
            }
        }

        Ok(())
    }

    /// Validate compatibility between weight and rebalancing strategies
    fn validate_strategy_compatibility(
        weight_config: &WeightStrategyConfig,
        rebalancing_config: &RebalancingStrategyConfig,
    ) -> StrategyResult<()> {
        // Check if both strategies are active
        if !weight_config.is_active || !rebalancing_config.is_active {
            return Err(StrategyError::StrategyPaused.into());
        }

        // Validate specific compatibility rules
        match (
            &weight_config.strategy_type,
            &rebalancing_config.strategy_type,
        ) {
            (WeightStrategyType::FixedWeight, RebalancingStrategyType::ThresholdBased) => {
                // Fixed weights with threshold rebalancing might not make sense
                // but we'll allow it for flexibility
            }
            (
                WeightStrategyType::VolatilityAdjusted,
                RebalancingStrategyType::VolatilityTriggered,
            ) => {
                // These work well together
            }
            _ => {
                // Most combinations are compatible
            }
        }

        Ok(())
    }

    /// Validate market cap weighted parameters
    fn validate_market_cap_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        // In a real implementation, deserialize and validate specific parameters
        // For now, just check basic constraints
        if params.len() > 64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate momentum weighted parameters
    fn validate_momentum_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        if params.len() > 64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate volatility adjusted parameters
    fn validate_volatility_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        if params.len() > 64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate fixed weight parameters
    fn validate_fixed_weight_params(params: &[u8], token_count: usize) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Check if parameters contain enough data for all tokens
        let expected_size = token_count * 8; // 8 bytes per u64 weight
        if params.len() < expected_size {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate that weights sum to 100%
        let mut total_weight = 0u64;
        for i in 0..token_count {
            let start_idx = i * 8;
            if start_idx + 8 <= params.len() {
                let weight_bytes: [u8; 8] = params[start_idx..start_idx + 8]
                    .try_into()
                    .map_err(|_| StrategyError::InvalidStrategyParameters)?;
                let weight = u64::from_le_bytes(weight_bytes);

                if weight > MAX_TOKEN_WEIGHT_BPS {
                    return Err(StrategyError::InvalidStrategyParameters.into());
                }

                total_weight += weight;
            }
        }

        if total_weight != BASIS_POINTS_MAX {
            return Err(StrategyError::InvalidWeightSum.into());
        }

        Ok(())
    }

    /// Validate technical indicator parameters
    fn validate_technical_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        if params.len() > 128 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate threshold-based rebalancing parameters
    fn validate_threshold_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        if params.len() > 32 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate time-based rebalancing parameters
    fn validate_time_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        if params.len() > 32 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate volatility-triggered rebalancing parameters
    fn validate_volatility_trigger_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        if params.len() > 32 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate drift-based rebalancing parameters
    fn validate_drift_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        if params.len() > 64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate hybrid rebalancing parameters
    fn validate_hybrid_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // Use defaults
        }

        if params.len() > 128 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }
}
