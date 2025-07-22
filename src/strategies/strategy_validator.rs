/*!
 * Strategy Validator Implementation
 *
 * Validation utilities for strategy configurations and parameters.
 */

use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// Strategy validator for comprehensive validation
pub struct StrategyValidator;

impl StrategyValidator {
    /// Validate complete strategy configuration
    pub fn validate_strategy_config(config: &StrategyConfig) -> StrategyResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Validate basic configuration
        if let Err(e) = Self::validate_basic_config(config) {
            result.add_error(ValidationError::Configuration(format!("{:?}", e)));
        }

        // Validate weight strategy
        if let Err(e) = Self::validate_weight_strategy(&config.weight_config) {
            result.add_error(ValidationError::WeightStrategy(format!("{:?}", e)));
        }

        // Validate rebalancing strategy
        if let Err(e) = Self::validate_rebalancing_strategy(&config.rebalancing_config) {
            result.add_error(ValidationError::RebalancingStrategy(format!("{:?}", e)));
        }

        // Validate optimization settings
        if let Err(e) = Self::validate_optimization_settings(&config.optimization_settings) {
            result.add_error(ValidationError::OptimizationSettings(format!("{:?}", e)));
        }

        // Validate risk settings
        if let Err(e) = Self::validate_risk_settings(&config.risk_settings) {
            result.add_error(ValidationError::RiskSettings(format!("{:?}", e)));
        }

        // Validate strategy compatibility
        if let Err(e) =
            Self::validate_strategy_compatibility(&config.weight_config, &config.rebalancing_config)
        {
            result.add_error(ValidationError::Compatibility(format!("{:?}", e)));
        }

        Ok(result)
    }

    /// Validate basic configuration fields
    fn validate_basic_config(config: &StrategyConfig) -> StrategyResult<()> {
        // Validate config ID
        if config.config_id == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate authority
        if config.authority == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate timestamps
        if config.created_at <= 0 || config.updated_at <= 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        if config.updated_at < config.created_at {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate weight strategy configuration
    fn validate_weight_strategy(config: &WeightStrategyConfig) -> StrategyResult<()> {
        // Validate token mints
        if config.token_mints.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        if config.token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }

        // Check for duplicate mints
        let mut seen = std::collections::HashSet::new();
        for mint in &config.token_mints {
            if *mint == Pubkey::default() {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
            if !seen.insert(*mint) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }

        // Validate parameters size
        if config.parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate strategy-specific parameters
        Self::validate_weight_strategy_parameters(
            &config.strategy_type,
            &config.parameters,
            config.token_mints.len(),
        )?;

        Ok(())
    }

    /// Validate rebalancing strategy configuration
    fn validate_rebalancing_strategy(config: &RebalancingStrategyConfig) -> StrategyResult<()> {
        // Validate parameters size
        if config.parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate timestamps
        if config.last_rebalance < 0 || config.next_rebalance < 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate strategy-specific parameters
        Self::validate_rebalancing_strategy_parameters(&config.strategy_type, &config.parameters)?;

        Ok(())
    }

    /// Validate optimization settings
    fn validate_optimization_settings(settings: &OptimizationSettings) -> StrategyResult<()> {
        // Validate batch size
        if settings.batch_size == 0 || settings.batch_size > MAX_BATCH_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate cache expiry
        if settings.cache_expiry == 0 || settings.cache_expiry > 86400 {
            // Max 24 hours
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate risk settings
    fn validate_risk_settings(settings: &RiskSettings) -> StrategyResult<()> {
        // Validate concentration limit
        if settings.max_concentration > MAX_CONCENTRATION_LIMIT_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate daily rebalance limit
        if settings.max_daily_rebalances == 0 || settings.max_daily_rebalances > 24 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate volatility circuit breaker
        if settings.volatility_circuit_breaker > 10000 {
            // Max 100%
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        Ok(())
    }

    /// Validate weight strategy parameters based on type
    fn validate_weight_strategy_parameters(
        strategy_type: &WeightStrategyType,
        parameters: &[u8],
        token_count: usize,
    ) -> StrategyResult<()> {
        match strategy_type {
            WeightStrategyType::EqualWeight => {
                // No parameters needed
                Ok(())
            }
            WeightStrategyType::MarketCapWeighted => {
                Self::validate_market_cap_parameters(parameters)
            }
            WeightStrategyType::MomentumWeighted => Self::validate_momentum_parameters(parameters),
            WeightStrategyType::VolatilityAdjusted => {
                Self::validate_volatility_parameters(parameters)
            }
            WeightStrategyType::FixedWeight => {
                Self::validate_fixed_weight_parameters(parameters, token_count)
            }
            WeightStrategyType::TechnicalIndicator => {
                Self::validate_technical_parameters(parameters)
            }
        }
    }

    /// Validate rebalancing strategy parameters based on type
    fn validate_rebalancing_strategy_parameters(
        strategy_type: &RebalancingStrategyType,
        parameters: &[u8],
    ) -> StrategyResult<()> {
        match strategy_type {
            RebalancingStrategyType::ThresholdBased => {
                Self::validate_threshold_parameters(parameters)
            }
            RebalancingStrategyType::TimeBased => Self::validate_time_based_parameters(parameters),
            RebalancingStrategyType::VolatilityTriggered => {
                Self::validate_volatility_triggered_parameters(parameters)
            }
            RebalancingStrategyType::DriftBased => {
                Self::validate_drift_based_parameters(parameters)
            }
            RebalancingStrategyType::Hybrid => Self::validate_hybrid_parameters(parameters),
        }
    }

    /// Validate strategy compatibility
    fn validate_strategy_compatibility(
        weight_config: &WeightStrategyConfig,
        rebalancing_config: &RebalancingStrategyConfig,
    ) -> StrategyResult<()> {
        // Both strategies must be active
        if !weight_config.is_active || !rebalancing_config.is_active {
            return Err(StrategyError::StrategyPaused.into());
        }

        // Check for incompatible combinations
        match (
            &weight_config.strategy_type,
            &rebalancing_config.strategy_type,
        ) {
            (WeightStrategyType::FixedWeight, RebalancingStrategyType::ThresholdBased) => {
                // This combination might not be very useful, but we'll allow it
            }
            _ => {
                // Most combinations are compatible
            }
        }

        Ok(())
    }

    // Parameter validation methods for specific strategy types
    fn validate_market_cap_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation - in production would deserialize and validate specific fields
        Ok(())
    }

    fn validate_momentum_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation
        Ok(())
    }

    fn validate_volatility_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation
        Ok(())
    }

    fn validate_fixed_weight_parameters(
        parameters: &[u8],
        token_count: usize,
    ) -> StrategyResult<()> {
        if parameters.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        let expected_size = token_count * 8; // 8 bytes per u64
        if parameters.len() < expected_size {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }

        // Validate weights sum to 100%
        let mut total_weight = 0u64;
        for i in 0..token_count {
            let start_idx = i * 8;
            if start_idx + 8 <= parameters.len() {
                let weight_bytes: [u8; 8] = parameters[start_idx..start_idx + 8]
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

    fn validate_technical_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation
        Ok(())
    }

    fn validate_threshold_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation
        Ok(())
    }

    fn validate_time_based_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation
        Ok(())
    }

    fn validate_volatility_triggered_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation
        Ok(())
    }

    fn validate_drift_based_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation
        Ok(())
    }

    fn validate_hybrid_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        // Simplified validation
        Ok(())
    }
}

/// Validation result structure
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationError {
    Configuration(String),
    WeightStrategy(String),
    RebalancingStrategy(String),
    OptimizationSettings(String),
    RiskSettings(String),
    Compatibility(String),
    Parameters(String),
}

/// Validation warning types
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    Performance(String),
    Compatibility(String),
    Configuration(String),
}
