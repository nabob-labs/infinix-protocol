//!
//! Strategy Factory Implementation
//!
//! 本模块实现策略工厂模式，用于创建和管理不同类型的策略配置，确保策略参数合规、可扩展、可插拔。

// 引入核心模块、适配器 trait、错误类型和策略模块。
use crate::core::*;
use crate::core::adapter::AdapterTrait;
use crate::error::StrategyError;
use crate::strategies::*;
use anchor_lang::prelude::*;

/// 策略工厂结构体，负责创建不同类型的策略配置。
pub struct StrategyFactory;

// 为策略工厂实现适配器 trait，便于统一注册和管理。
impl AdapterTrait for StrategyFactory {
    /// 返回适配器名称。
    fn name(&self) -> &'static str { "strategy_factory" }
    /// 返回适配器版本。
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表。
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器当前状态。
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

// 使用构造器自动注册策略工厂到全局工厂。
#[ctor::ctor]
fn auto_register_strategy_factory() {
    // 实例化策略工厂。
    let adapter = StrategyFactory;
    // 获取全局适配器工厂的可变引用。
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    // 注册适配器。
    factory.register(adapter);
}

impl StrategyFactory {
    /// 创建权重策略配置，根据类型和参数。
    ///
    /// # 参数
    /// * `strategy_type` - 权重策略类型。
    /// * `parameters` - 策略参数字节数组。
    /// * `token_mints` - 资产 mint 列表。
    /// # 返回
    /// * 权重策略配置或错误。
    pub fn create_weight_strategy(
        strategy_type: WeightStrategyType,
        parameters: &[u8],
        token_mints: &[Pubkey],
    ) -> StrategyResult<WeightStrategyConfig> {
        // 校验资产数量边界。
        if token_mints.is_empty() || token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        // 校验参数长度。
        if parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 获取当前时间戳。
        let current_time = Clock::get()?.unix_timestamp;
        // 构建权重策略配置。
        let config = WeightStrategyConfig {
            strategy_type,
            parameters: parameters.to_vec(),
            token_mints: token_mints.to_vec(),
            is_active: true,
            last_calculation: 0,
        };
        // 校验配置合规性。
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
        // 校验参数长度。
        if parameters.len() > MAX_STRATEGY_PARAMETERS_SIZE {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 校验再平衡阈值。
        if rebalancing_threshold == 0 || rebalancing_threshold > MAX_REBALANCE_THRESHOLD_BPS {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 校验最小间隔。
        if min_interval < MIN_REBALANCE_INTERVAL {
            return Err(StrategyError::InvalidTimeWindow.into());
        }
        // 获取当前时间戳。
        let current_time = Clock::get()?.unix_timestamp;
        // 构建再平衡策略配置。
        let config = RebalancingStrategyConfig {
            strategy_type,
            parameters: parameters.to_vec(),
            is_active: true,
            last_rebalance: 0,
            next_rebalance: current_time + min_interval as i64,
        };
        // 校验配置合规性。
        Self::validate_rebalancing_strategy_config(&config)?;
        Ok(config)
    }
    /// 创建默认优化设置。
    pub fn create_optimization_settings() -> OptimizationSettings {
        OptimizationSettings {
            enable_caching: true,
            enable_parallel: false,
            batch_size: DEFAULT_BATCH_SIZE,
            cache_expiry: DEFAULT_CACHE_TTL as u64,
        }
    }
    /// 创建默认风险设置。
    pub fn create_risk_settings() -> RiskSettings {
        RiskSettings {
            max_concentration: DEFAULT_CONCENTRATION_LIMIT_BPS,
            max_daily_rebalances: 4,
            enable_circuit_breakers: true,
            volatility_circuit_breaker: CIRCUIT_BREAKER_THRESHOLD_BPS,
        }
    }
    /// 创建完整策略配置。
    pub fn create_strategy_config(
        config_id: u64,
        authority: Pubkey,
        weight_config: WeightStrategyConfig,
        rebalancing_config: RebalancingStrategyConfig,
    ) -> StrategyResult<StrategyConfig> {
        // 校验权重与再平衡策略兼容性。
        Self::validate_strategy_compatibility(&weight_config, &rebalancing_config)?;
        // 构建策略配置。
        let config = StrategyConfig::new(config_id, authority, weight_config, rebalancing_config)?;
        Ok(config)
    }
    /// 校验权重策略配置合规性。
    fn validate_weight_strategy_config(config: &WeightStrategyConfig) -> StrategyResult<()> {
        // 校验资产数量。
        if config.token_mints.is_empty() || config.token_mints.len() > MAX_TOKENS {
            return Err(StrategyError::InvalidTokenCount.into());
        }
        // 检查资产 mint 是否重复。
        let mut seen = std::collections::HashSet::new();
        for mint in &config.token_mints {
            if !seen.insert(*mint) {
                return Err(StrategyError::InvalidStrategyParameters.into());
            }
        }
        // 校验策略类型参数。
        match config.strategy_type {
            WeightStrategyType::EqualWeight => {
                // 无需额外校验。
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
    /// 校验再平衡策略配置合规性。
    fn validate_rebalancing_strategy_config(
        config: &RebalancingStrategyConfig,
    ) -> StrategyResult<()> {
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
    /// 校验权重与再平衡策略兼容性。
    fn validate_strategy_compatibility(
        weight_config: &WeightStrategyConfig,
        rebalancing_config: &RebalancingStrategyConfig,
    ) -> StrategyResult<()> {
        // 检查策略是否激活。
        if !weight_config.is_active || !rebalancing_config.is_active {
            return Err(StrategyError::StrategyPaused.into());
        }
        // 兼容性规则。
        match (
            &weight_config.strategy_type,
            &rebalancing_config.strategy_type,
        ) {
            (WeightStrategyType::FixedWeight, RebalancingStrategyType::ThresholdBased) => {
                // 固定权重与阈值再平衡允许但需谨慎。
            }
            (
                WeightStrategyType::VolatilityAdjusted,
                RebalancingStrategyType::VolatilityTriggered,
            ) => {
                // 波动率相关策略兼容。
            }
            _ => {
                // 其他组合默认兼容。
            }
        }
        Ok(())
    }
    /// 校验市值加权参数。
    fn validate_market_cap_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(()); // 使用默认值。
        }
        if params.len() > 64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验动量加权参数。
    fn validate_momentum_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(());
        }
        if params.len() > 64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验波动率加权参数。
    fn validate_volatility_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(());
        }
        if params.len() > 64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验固定权重参数。
    fn validate_fixed_weight_params(params: &[u8], token_count: usize) -> StrategyResult<()> {
        if params.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let expected_size = token_count * 8; // 每个权重 8 字节
        if params.len() < expected_size {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 校验权重总和为 10000。
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
    /// 校验技术指标参数。
    fn validate_technical_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(());
        }
        if params.len() > 128 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验阈值再平衡参数。
    fn validate_threshold_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(());
        }
        if params.len() > 32 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验时间再平衡参数。
    fn validate_time_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(());
        }
        if params.len() > 32 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验波动率触发再平衡参数。
    fn validate_volatility_trigger_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(());
        }
        if params.len() > 32 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验漂移再平衡参数。
    fn validate_drift_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(());
        }
        if params.len() > 64 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 校验混合再平衡参数。
    fn validate_hybrid_params(params: &[u8]) -> StrategyResult<()> {
        if params.is_empty() {
            return Ok(());
        }
        if params.len() > 128 {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
    /// 移除权重策略（可实现为全局注册表移除）。
    pub fn remove_weight_strategy(strategy_type: WeightStrategyType) {
        // 可实现为全局注册表移除
    }
    /// 列出所有支持的权重策略类型。
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
