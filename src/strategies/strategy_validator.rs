//!
//! Strategy Validator Implementation
//!
//! 本模块实现策略配置与参数的合规性校验工具，确保所有策略实例、参数、配置、风险与优化设置均符合业务和安全要求。

// 引入核心模块、错误类型、策略模块和 Anchor 依赖。
use crate::core::*;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 策略校验器结构体，提供全量合规性校验。
pub struct StrategyValidator;

impl StrategyValidator {
    /// 校验完整策略配置，包括基础字段、权重策略、再平衡策略、优化与风险设置、兼容性。
    pub fn validate_strategy_config(config: &StrategyConfig) -> StrategyResult<ValidationResult> {
        let mut result = ValidationResult::new();
        // 校验基础配置。
        if let Err(e) = Self::validate_basic_config(config) {
            result.add_error(ValidationError::Configuration(format!("{:?}", e)));
        }
        // 校验权重策略。
        if let Err(e) = Self::validate_weight_strategy(&config.weight_config) {
            result.add_error(ValidationError::WeightStrategy(format!("{:?}", e)));
        }
        // 校验再平衡策略。
        if let Err(e) = Self::validate_rebalancing_strategy(&config.rebalancing_config) {
            result.add_error(ValidationError::RebalancingStrategy(format!("{:?}", e)));
        }
        // 校验优化设置。
        if let Err(e) = Self::validate_optimization_settings(&config.optimization_settings) {
            result.add_error(ValidationError::OptimizationSettings(format!("{:?}", e)));
        }
        // 校验风险设置。
        if let Err(e) = Self::validate_risk_settings(&config.risk_settings) {
            result.add_error(ValidationError::RiskSettings(format!("{:?}", e)));
        }
        // 校验策略兼容性。
        if let Err(e) =
            Self::validate_strategy_compatibility(&config.weight_config, &config.rebalancing_config)
        {
            result.add_error(ValidationError::Compatibility(format!("{:?}", e)));
        }
        Ok(result)
    }
    /// 校验基础配置字段。
    fn validate_basic_config(config: &StrategyConfig) -> StrategyResult<()> {
        // 配置 ID 必须非零。
        if config.config_id == 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 权限必须有效。
        if config.authority == Pubkey::default() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 时间戳必须有效。
        if config.created_at <= 0 || config.updated_at <= 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        if config.updated_at < config.created_at {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验权重策略配置。
    fn validate_weight_strategy(config: &WeightStrategyConfig) -> StrategyResult<()> {
        // 资产 mint 必须非空且不超限。
        if config.token_mints.is_empty() {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if config.token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        // 检查 mint 是否重复或无效。
        let mut seen = std::collections::HashSet::new();
        for mint in &config.token_mints {
            if *mint == Pubkey::default() {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
            if !seen.insert(*mint) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        // 参数长度校验。
        if config.parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 校验策略类型参数。
        Self::validate_weight_strategy_parameters(
            &config.strategy_type,
            &config.parameters,
            config.token_mints.len(),
        )?;
        Ok(())
    }
    /// 校验再平衡策略配置。
    fn validate_rebalancing_strategy(config: &RebalancingStrategyConfig) -> StrategyResult<()> {
        // 参数长度校验。
        if config.parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 时间戳校验。
        if config.last_rebalance < 0 || config.next_rebalance < 0 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 校验策略类型参数。
        Self::validate_rebalancing_strategy_parameters(&config.strategy_type, &config.parameters)?;
        Ok(())
    }
    /// 校验优化设置。
    fn validate_optimization_settings(settings: &OptimizationSettings) -> StrategyResult<()> {
        // 批处理大小校验。
        if settings.batch_size == 0 || settings.batch_size > MAX_BATCH_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 缓存过期时间校验。
        if settings.cache_expiry == 0 || settings.cache_expiry > 86400 {
            // 最长 24 小时
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验风险设置。
    fn validate_risk_settings(settings: &RiskSettings) -> StrategyResult<()> {
        // 集中度上限校验。
        if settings.max_concentration > MAX_CONCENTRATION_LIMIT_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 每日再平衡次数校验。
        if settings.max_daily_rebalances == 0 || settings.max_daily_rebalances > 24 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 断路器波动率阈值校验。
        if settings.volatility_circuit_breaker > 10000 {
            // 最大 100%
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 按类型校验权重策略参数。
    fn validate_weight_strategy_parameters(
        strategy_type: &WeightStrategyType,
        parameters: &[u8],
        token_count: usize,
    ) -> StrategyResult<()> {
        match strategy_type {
            WeightStrategyType::EqualWeight => {
                // 无需参数
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
    /// 按类型校验再平衡策略参数。
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
    /// 校验策略兼容性。
    fn validate_strategy_compatibility(
        weight_config: &WeightStrategyConfig,
        rebalancing_config: &RebalancingStrategyConfig,
    ) -> StrategyResult<()> {
        // 策略必须均为激活状态。
        if !weight_config.is_active || !rebalancing_config.is_active {
            return Err(StrategyError::StrategyPaused.into());
        }
        // 检查不兼容组合。
        match (
            &weight_config.strategy_type,
            &rebalancing_config.strategy_type,
        ) {
            (WeightStrategyType::FixedWeight, RebalancingStrategyType::ThresholdBased) => {
                // 允许但需谨慎。
            }
            _ => {
                // 其他组合默认兼容。
            }
        }
        Ok(())
    }
    // 各类型参数校验方法（简化实现）。
    fn validate_market_cap_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        Ok(())
    }
    fn validate_momentum_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        Ok(())
    }
    fn validate_volatility_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        Ok(())
    }
    fn validate_fixed_weight_parameters(
        parameters: &[u8],
        token_count: usize,
    ) -> StrategyResult<()> {
        if parameters.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let expected_size = token_count * 8; // 每个权重 8 字节
        if parameters.len() < expected_size {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 校验权重总和为 10000。
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
        Ok(())
    }
    fn validate_threshold_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        Ok(())
    }
    fn validate_time_based_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        Ok(())
    }
    fn validate_volatility_triggered_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        Ok(())
    }
    fn validate_drift_based_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        Ok(())
    }
    fn validate_hybrid_parameters(_parameters: &[u8]) -> StrategyResult<()> {
        Ok(())
    }
}

/// 校验结果结构体，包含有效性、错误与警告列表。
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,                // 是否有效
    pub errors: Vec<ValidationError>,  // 错误列表
    pub warnings: Vec<ValidationWarning>, // 警告列表
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

/// 校验错误类型枚举。
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

/// 校验警告类型枚举。
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    Performance(String),
    Compatibility(String),
    Configuration(String),
}
