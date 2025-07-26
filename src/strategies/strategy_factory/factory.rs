//!
//! factory.rs - 策略工厂实现
//!
//! 本文件实现StrategyFactory结构体及其所有策略创建、校验、管理方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::core::*;
use crate::core::adapter::AdapterTrait;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 策略工厂结构体，负责创建不同类型的策略配置。
pub struct StrategyFactory;

impl StrategyFactory {
    /// 创建权重策略配置，根据类型和参数。
    pub fn create_weight_strategy(
        strategy_type: WeightStrategyType,
        parameters: &[u8],
        token_mints: &[Pubkey],
    ) -> StrategyResult<WeightStrategyConfig> {
        if token_mints.is_empty() || token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let config = WeightStrategyConfig {
            strategy_type,
            parameters: parameters.to_vec(),
            token_mints: token_mints.to_vec(),
            is_active: true,
            last_calculation: 0,
        };
        Self::validate_weight_strategy_config(&config)?;
        Ok(config)
    }
    /// 创建再平衡策略配置。
    pub fn create_rebalancing_strategy(
        strategy_type: RebalancingStrategyType,
        parameters: &[u8],
        rebalancing_threshold: u64,
        min_interval: u64,
    ) -> StrategyResult<RebalancingStrategyConfig> {
        if parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        if rebalancing_threshold == 0 || rebalancing_threshold > MAX_REBALANCE_THRESHOLD_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        if min_interval < MIN_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        let config = RebalancingStrategyConfig {
            strategy_type,
            parameters: parameters.to_vec(),
            is_active: true,
            last_rebalance: 0,
            next_rebalance: 0,
        };
        Self::validate_rebalancing_strategy_config(&config)?;
        Ok(config)
    }
    /// 创建性能优化设置。
    pub fn create_optimization_settings() -> OptimizationSettings {
        OptimizationSettings::default()
    }
    /// 创建风险管理设置。
    pub fn create_risk_settings() -> RiskSettings {
        RiskSettings::default()
    }
    /// 创建完整策略配置。
    pub fn create_strategy_config(
        config_id: u64,
        authority: Pubkey,
        weight_config: WeightStrategyConfig,
        rebalancing_config: RebalancingStrategyConfig,
    ) -> StrategyResult<StrategyConfig> {
        let config = StrategyConfig::new(
            config_id,
            authority,
            weight_config,
            rebalancing_config,
        )?;
        Ok(config)
    }
    /// 校验权重策略配置。
    fn validate_weight_strategy_config(config: &WeightStrategyConfig) -> StrategyResult<()> {
        if config.token_mints.is_empty() || config.token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        if config.parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验再平衡策略配置。
    fn validate_rebalancing_strategy_config(config: &RebalancingStrategyConfig) -> StrategyResult<()> {
        if config.parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验策略兼容性。
    fn validate_strategy_compatibility(
        weight_config: &WeightStrategyConfig,
        rebalancing_config: &RebalancingStrategyConfig,
    ) -> StrategyResult<()> {
        if !weight_config.is_active || !rebalancing_config.is_active {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验市值加权参数。
    fn validate_market_cap_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验动量参数。
    fn validate_momentum_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验波动率参数。
    fn validate_volatility_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验固定权重参数。
    fn validate_fixed_weight_params(params: &[u8], token_count: usize) -> StrategyResult<()> {
        if params.len() != token_count {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验技术指标参数。
    fn validate_technical_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验阈值参数。
    fn validate_threshold_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验定时参数。
    fn validate_time_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验波动率触发参数。
    fn validate_volatility_trigger_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验漂移参数。
    fn validate_drift_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验混合参数。
    fn validate_hybrid_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 移除权重策略（示例）。
    pub fn remove_weight_strategy(_strategy_type: WeightStrategyType) {
        // 实际实现应从注册表移除
    }
    /// 列出所有权重策略类型（示例）。
    pub fn list_weight_strategies() -> Vec<WeightStrategyType> {
        vec![
            WeightStrategyType::EqualWeight,
            WeightStrategyType::MarketCapWeighted,
            WeightStrategyType::MomentumWeighted,
            WeightStrategyType::VolatilityAdjusted,
            WeightStrategyType::FixedWeight,
            WeightStrategyType::TechnicalIndicator,
        ]
    }
} 