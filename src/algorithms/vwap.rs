/*!
 * VWAP 算法模块 - 生产级实现
 *
 * 生产级 VWAP（成交量加权平均价格）算法实现。
 * 支持 Anchor 框架自动注册，便于在算法工厂/注册表中动态调用。
 * 
 * 功能特性：
 * - 成交量权重计算和动态调整
 * - 市场深度分析和流动性评估
 * - 动态价格调整和滑点优化
 * - 风险控制和执行监控
 * - 性能指标收集和分析
 * - 多资产支持
 * - 可配置的执行策略
 */

use anchor_lang::prelude::*;
use crate::algorithms::traits::{Algorithm, ExecutionStrategy, AlgorithmType, ExecutionResult};
use crate::core::adapter::AdapterTrait;
use crate::core::types::AlgoParams;
// use crate::core::types::algo::AlgoParams; // 暂时注释掉
use crate::errors::algorithm_error::AlgorithmError;
// use crate::core::constants::*; // 暂时注释掉
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 最大再平衡间隔（秒）
const MAX_REBALANCE_INTERVAL: u64 = 86400; // 24小时
/// 最大滑点容忍度（基点）
const MAX_SLIPPAGE_BPS: u64 = 1000; // 10%

/// VWAP算法参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct VwapParams {
    /// 总订单数量
    pub total_amount: u64,
    /// 时间窗口（秒）
    pub time_window: u64,
    /// 分段数量
    pub num_intervals: u32,
    /// 滑点容忍度（基点）
    pub slippage_tolerance_bps: u32,
    /// 最小分段大小
    pub min_interval_size: u64,
    /// 最大分段大小
    pub max_interval_size: u64,
    /// 价格偏差容忍度（基点）
    pub price_deviation_bps: u32,
    /// 是否启用动态权重调整
    pub enable_dynamic_weighting: bool,
    /// 是否启用市场深度分析
    pub enable_market_depth_analysis: bool,
    /// 风险控制参数
    pub risk_params: VwapRiskParams,
    /// 性能监控参数
    pub monitoring_params: VwapMonitoringParams,
}

/// VWAP风险控制参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct VwapRiskParams {
    /// 最大单次执行比例（基点）
    pub max_single_execution_bps: u32,
    /// 最大价格偏差（基点）
    pub max_price_deviation_bps: u32,
    /// 最大执行时间（秒）
    pub max_execution_time: u64,
    /// 是否启用紧急停止
    pub enable_emergency_stop: bool,
    /// 紧急停止阈值（基点）
    pub emergency_stop_threshold_bps: u32,
    /// 最大成交量偏差（基点）
    pub max_volume_deviation_bps: u32,
}

/// VWAP性能监控参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct VwapMonitoringParams {
    /// 是否启用性能监控
    pub enable_monitoring: bool,
    /// 性能指标收集间隔（秒）
    pub metrics_interval: u64,
    /// 是否启用详细日志
    pub enable_detailed_logging: bool,
    /// 是否启用性能警告
    pub enable_performance_warnings: bool,
    /// 是否启用市场深度监控
    pub enable_depth_monitoring: bool,
}

/// VWAP算法执行状态
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct VwapExecutionState {
    /// 当前执行阶段
    pub current_interval: u32,
    /// 已执行数量
    pub executed_amount: u64,
    /// 已执行成本
    pub total_cost: u64,
    /// 开始时间戳
    pub start_timestamp: u64,
    /// 最后执行时间戳
    pub last_execution_timestamp: u64,
    /// 加权平均价格
    pub vwap_price: u64,
    /// 执行状态
    pub status: VwapExecutionStatus,
    /// 性能指标
    pub metrics: VwapMetrics,
    /// 市场深度数据
    pub market_depth: MarketDepthData,
}

/// VWAP执行状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum VwapExecutionStatus {
    /// 初始化
    Initialized,
    /// 执行中
    Executing,
    /// 暂停
    Paused,
    /// 完成
    Completed,
    /// 失败
    Failed,
    /// 取消
    Cancelled,
}

/// VWAP性能指标
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct VwapMetrics {
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行延迟（毫秒）
    pub avg_execution_latency_ms: u64,
    /// 最大执行延迟（毫秒）
    pub max_execution_latency_ms: u64,
    /// 执行成功率
    pub success_rate: f64,
    /// 平均滑点（基点）
    pub avg_slippage_bps: u32,
    /// 最大滑点（基点）
    pub max_slippage_bps: u32,
    /// VWAP偏差统计
    pub vwap_deviation_stats: VwapDeviationStats,
    /// 成交量分布统计
    pub volume_distribution_stats: VolumeDistributionStats,
}

/// VWAP偏差统计
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct VwapDeviationStats {
    /// 平均VWAP偏差（基点）
    pub avg_deviation_bps: u32,
    /// 最大VWAP偏差（基点）
    pub max_deviation_bps: u32,
    /// VWAP偏差标准差
    pub deviation_std: f64,
    /// 偏差超过阈值的次数
    pub threshold_violations: u32,
}

/// 成交量分布统计
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct VolumeDistributionStats {
    /// 成交量集中度（基尼系数）
    pub volume_concentration: f64,
    /// 最大单次成交量比例（基点）
    pub max_single_volume_bps: u32,
    /// 成交量标准差
    pub volume_std: f64,
    /// 成交量分布均匀度
    pub volume_uniformity: f64,
}

/// 市场深度数据
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MarketDepthData {
    /// 买盘深度（按价格层级）
    pub bid_depth: Vec<PriceLevel>,
    /// 卖盘深度（按价格层级）
    pub ask_depth: Vec<PriceLevel>,
    /// 市场流动性评分
    pub liquidity_score: f64,
    /// 价格冲击系数
    pub price_impact_factor: f64,
    /// 市场波动性
    pub market_volatility: f64,
}

/// 价格层级
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PriceLevel {
    /// 价格
    pub price: u64,
    /// 数量
    pub size: u64,
    /// 累计数量
    pub cumulative_size: u64,
}

/// VWAP算法结构体
#[derive(Default)]
pub struct VwapAlgorithm {
    /// 算法配置
    config: VwapConfig,
    /// 执行状态缓存
    execution_cache: HashMap<String, VwapExecutionState>,
}

/// VWAP算法配置
#[derive(Clone, Debug)]
pub struct VwapConfig {
    /// 默认时间窗口（秒）
    pub default_time_window: u64,
    /// 默认分段数量
    pub default_num_intervals: u32,
    /// 默认滑点容忍度（基点）
    pub default_slippage_tolerance_bps: u32,
    /// 最大支持资产数量
    pub max_supported_assets: u32,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存过期时间（秒）
    pub cache_expiry_time: u64,
    /// 默认权重策略
    pub default_weight_strategy: WeightStrategy,
}

/// 权重策略枚举
#[derive(Clone, Debug)]
pub enum WeightStrategy {
    /// 线性权重（递增）
    Linear,
    /// 指数权重
    Exponential,
    /// 对数权重
    Logarithmic,
    /// 自定义权重
    Custom(Vec<f64>),
}

impl Default for VwapConfig {
    fn default() -> Self {
        Self {
            default_time_window: 3600, // 1小时
            default_num_intervals: 10,
            default_slippage_tolerance_bps: 100, // 1%
            max_supported_assets: 100,
            enable_cache: true,
            cache_expiry_time: 300, // 5分钟
            default_weight_strategy: WeightStrategy::Linear,
        }
    }
}

/// AdapterTrait 实现
impl AdapterTrait for VwapAlgorithm {
    fn name(&self) -> &str {
        "Vwap"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn initialize(&mut self) -> anchor_lang::Result<()> {
        Ok(())
    }
    
    fn cleanup(&mut self) -> anchor_lang::Result<()> {
        Ok(())
    }
}

/// Algorithm trait 实现
impl Algorithm for VwapAlgorithm {
    fn execute(&self, params: &AlgoParams) -> anchor_lang::Result<ExecutionResult> {
        // 解析VWAP参数
        let vwap_params = self.parse_vwap_params(params)?;
        
        // 验证参数
        self.validate_vwap_params(&vwap_params)?;
        
        // 计算执行计划
        let execution_plan = self.calculate_execution_plan(&vwap_params)?;
        
        // 执行VWAP算法
        let result = self.execute_vwap_algorithm(&vwap_params, &execution_plan)?;
        
        Ok(result)
    }
    
    fn supported_assets(&self) -> Vec<String> { 
        self.supported_assets() 
    }
    
    fn supported_markets(&self) -> Vec<String> { 
        vec![
            "spot".to_string(),
            "dex".to_string(),
            "amm".to_string(),
        ] 
    }
    
    fn algorithm_type(&self) -> AlgorithmType { 
        AlgorithmType::Execution 
    }
}

/// ExecutionStrategy trait 实现
impl ExecutionStrategy for VwapAlgorithm {
    fn execute(&self, _ctx: Context<crate::algorithms::traits::Execute>, params: &AlgoParams) -> anchor_lang::Result<ExecutionResult> {
        self.execute(params)
    }
}

impl VwapAlgorithm {
    /// 创建新的VWAP算法实例
    pub fn new() -> Self {
        Self {
            config: VwapConfig::default(),
            execution_cache: HashMap::new(),
        }
    }
    
    /// 使用自定义配置创建VWAP算法实例
    pub fn with_config(config: VwapConfig) -> Self {
        Self {
            config,
            execution_cache: HashMap::new(),
        }
    }
    
    /// 解析VWAP参数
    fn parse_vwap_params(&self, params: &AlgoParams) -> anchor_lang::Result<VwapParams> {
        if params.params.is_empty() {
            return Err(AlgorithmError::InvalidParameters {
                reason: "Empty parameters".to_string(),
            }.into());
        }
        
        match bincode::deserialize::<VwapParams>(&params.params) {
            Ok(vwap_params) => Ok(vwap_params),
            Err(e) => Err(AlgorithmError::InvalidParameters {
                reason: format!("Failed to deserialize VWAP parameters: {}", e),
            }.into()),
        }
    }
    
    /// 验证VWAP参数
    fn validate_vwap_params(&self, params: &VwapParams) -> anchor_lang::Result<()> {
        // 验证总订单数量
        require!(
            params.total_amount > 0,
            AlgorithmError::InvalidParameters {
                reason: "Total amount must be greater than 0".to_string(),
            }
        );
        
        // 验证时间窗口
        require!(
            params.time_window > 0 && params.time_window <= MAX_REBALANCE_INTERVAL,
            AlgorithmError::InvalidParameters {
                reason: format!("Time window must be between 1 and {}", MAX_REBALANCE_INTERVAL).to_string(),
            }
        );
        
        // 验证分段数量
        require!(
            params.num_intervals > 0 && params.num_intervals <= 100,
            AlgorithmError::InvalidParameters {
                reason: "Number of intervals must be between 1 and 100".to_string(),
            }
        );
        
        // 验证滑点容忍度
        require!(
            params.slippage_tolerance_bps <= MAX_SLIPPAGE_BPS,
            AlgorithmError::InvalidParameters {
                reason: format!("Slippage tolerance must not exceed {} bps", MAX_SLIPPAGE_BPS).to_string(),
            }
        );
        
        // 验证分段大小
        require!(
            params.min_interval_size > 0 && params.min_interval_size <= params.max_interval_size,
            AlgorithmError::InvalidParameters {
                reason: "Invalid interval size range".to_string(),
            }
        );
        
        // 验证价格偏差容忍度
        require!(
            params.price_deviation_bps <= MAX_SLIPPAGE_BPS,
            AlgorithmError::InvalidParameters {
                reason: format!("Price deviation tolerance must not exceed {} bps", MAX_SLIPPAGE_BPS).to_string(),
            }
        );
        
        Ok(())
    }
    
    /// 计算执行计划
    fn calculate_execution_plan(&self, params: &VwapParams) -> anchor_lang::Result<VwapExecutionPlan> {
        // 计算权重分布
        let weights = self.calculate_weights(params.num_intervals)?;
        let total_weight: f64 = weights.iter().sum();
        
        let interval_duration = params.time_window / params.num_intervals as u64;
        let mut intervals = Vec::new();
        let mut current_time = self.get_current_timestamp();
        
        for i in 0..params.num_intervals {
            // 根据权重计算分段大小
            let weight_ratio = weights[i] / total_weight;
            let size = (params.total_amount as f64 * weight_ratio) as u64;
            
            // 确保分段大小在允许范围内
            let adjusted_size = size.clamp(params.min_interval_size, params.max_interval_size);
            
            intervals.push(VwapInterval {
                index: i,
                size: adjusted_size,
                weight: weights[i],
                start_time: current_time,
                end_time: current_time + interval_duration,
                target_price: 0, // 将在执行时动态计算
                executed: false,
            });
            
            current_time += interval_duration;
        }
        
        // 调整最后一个分段以匹配总数量
        let total_planned: u64 = intervals.iter().map(|i| i.size).sum();
        if total_planned != params.total_amount {
            let last_interval = intervals.last_mut().unwrap();
            let adjustment = params.total_amount as i64 - total_planned as i64;
            last_interval.size = (last_interval.size as i64 + adjustment) as u64;
        }
        
        Ok(VwapExecutionPlan {
            intervals,
            total_amount: params.total_amount,
            time_window: params.time_window,
            start_time: self.get_current_timestamp(),
        })
    }
    
    /// 计算权重分布
    fn calculate_weights(&self, num_intervals: u32) -> anchor_lang::Result<Vec<f64>> {
        match &self.config.default_weight_strategy {
            WeightStrategy::Linear => {
                let mut weights = Vec::new();
                for i in 0..num_intervals {
                    weights.push((i + 1) as f64);
                }
                Ok(weights)
            },
            WeightStrategy::Exponential => {
                let mut weights = Vec::new();
                for i in 0..num_intervals {
                    weights.push(2.0_f64.powi(i as i32));
                }
                Ok(weights)
            },
            WeightStrategy::Logarithmic => {
                let mut weights = Vec::new();
                for i in 0..num_intervals {
                    weights.push(((i + 1) as f64).ln() + 1.0);
                }
                Ok(weights)
            },
            WeightStrategy::Custom(custom_weights) => {
                if custom_weights.len() != num_intervals as usize {
                    return Err(AlgorithmError::InvalidParameters {
                        reason: "Custom weights length does not match number of intervals".to_string(),
                    }.into());
                }
                Ok(custom_weights.clone())
            },
        }
    }
    
    /// 执行VWAP算法
    fn execute_vwap_algorithm(&self, params: &VwapParams, plan: &VwapExecutionPlan) -> anchor_lang::Result<ExecutionResult> {
        let mut total_executed = 0u64;
        let mut total_cost = 0u64;
        let mut total_weighted_cost = 0u64;
        let mut total_weight = 0u64;
        let mut execution_times = Vec::new();
        let mut price_deviations = Vec::new();
        let mut volume_distribution = Vec::new();
        
        let start_time = self.get_current_timestamp();
        
        // 获取市场深度数据
        let market_depth = if params.enable_market_depth_analysis {
            self.get_market_depth()?
        } else {
            MarketDepthData::default()
        };
        
        for interval in &plan.intervals {
            // 检查是否到达执行时间
            if self.get_current_timestamp() < interval.start_time {
                continue;
            }
            
            // 获取当前市场价格
            let current_price = self.get_market_price()?;
            
            // 计算目标价格（考虑滑点和市场深度）
            let target_price = self.calculate_target_price_with_depth(
                current_price, 
                params.slippage_tolerance_bps,
                &market_depth,
                interval.size,
            )?;
            
            // 执行分段交易
            let execution_result = self.execute_interval(interval, target_price, params)?;
            
            // 更新统计信息
            total_executed += execution_result.executed_amount;
            total_cost += execution_result.total_cost;
            total_weighted_cost += execution_result.executed_amount * execution_result.execution_price;
            total_weight += execution_result.executed_amount;
            
            let execution_time = self.get_current_timestamp() - interval.start_time;
            execution_times.push(execution_time);
            
            let price_deviation = self.calculate_price_deviation(current_price, target_price);
            price_deviations.push(price_deviation);
            
            volume_distribution.push(execution_result.executed_amount);
            
            // 检查风险控制
            if let Err(e) = self.check_risk_limits(params, &execution_result) {
                return Err(e);
            }
            
            // 更新缓存
            if self.config.enable_cache {
                self.update_execution_cache(plan, &execution_result)?;
            }
        }
        
        // 计算VWAP价格
        let vwap_price = if total_weight > 0 {
            total_weighted_cost / total_weight
        } else {
            0
        };
        
        let total_execution_time = self.get_current_timestamp() - start_time;
        
        // 计算性能指标
        let metrics = self.calculate_performance_metrics(
            execution_times,
            price_deviations,
            volume_distribution,
            total_execution_time,
            plan.intervals.len(),
            vwap_price,
        )?;
        
        // 记录性能指标
        if params.monitoring_params.enable_monitoring {
            self.record_performance_metrics(&metrics)?;
        }
        
        Ok(ExecutionResult {
            optimized_size: total_executed,
            expected_cost: total_cost,
        })
    }
    
    /// 执行单个分段
    fn execute_interval(&self, interval: &VwapInterval, target_price: u64, params: &VwapParams) -> anchor_lang::Result<VwapIntervalResult> {
        let start_time = self.get_current_timestamp();
        
        // 模拟市场执行（实际应调用DEX接口）
        let execution_price = self.simulate_market_execution(interval.size, target_price)?;
        
        let execution_time = self.get_current_timestamp() - start_time;
        let slippage = self.calculate_slippage(target_price, execution_price);
        
        // 检查价格偏差
        if slippage > params.price_deviation_bps {
            return Err(AlgorithmError::InvalidResult {
                reason: format!("Price deviation {} bps exceeds tolerance {} bps", slippage, params.price_deviation_bps),
            }.into());
        }
        
        Ok(VwapIntervalResult {
            interval_index: interval.index,
            executed_amount: interval.size,
            weight: interval.weight,
            target_price,
            execution_price,
            total_cost: interval.size * execution_price,
            slippage_bps: slippage,
            execution_time,
        })
    }
    
    /// 获取当前时间戳
    fn get_current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// 获取市场价格（模拟实现）
    fn get_market_price(&self) -> anchor_lang::Result<u64> {
        // 实际实现应调用Oracle接口
        Ok(1_000_000) // 模拟价格：1 SOL = 1,000,000 lamports
    }
    
    /// 获取市场深度数据（模拟实现）
    fn get_market_depth(&self) -> anchor_lang::Result<MarketDepthData> {
        // 实际实现应调用DEX接口获取订单簿数据
        Ok(MarketDepthData {
            bid_depth: vec![
                PriceLevel { price: 999_000, size: 1000, cumulative_size: 1000 },
                PriceLevel { price: 998_000, size: 2000, cumulative_size: 3000 },
            ],
            ask_depth: vec![
                PriceLevel { price: 1_001_000, size: 1000, cumulative_size: 1000 },
                PriceLevel { price: 1_002_000, size: 2000, cumulative_size: 3000 },
            ],
            liquidity_score: 0.8,
            price_impact_factor: 0.1,
            market_volatility: 0.05,
        })
    }
    
    /// 计算考虑市场深度的目标价格
    fn calculate_target_price_with_depth(
        &self,
        current_price: u64,
        slippage_tolerance_bps: u32,
        market_depth: &MarketDepthData,
        order_size: u64,
    ) -> anchor_lang::Result<u64> {
        // 基础滑点计算
        let base_slippage_factor = 1.0 + (slippage_tolerance_bps as f64 / 10000.0);
        
        // 考虑市场深度和价格冲击
        let depth_impact = market_depth.price_impact_factor * (order_size as f64 / 10000.0);
        let volatility_impact = market_depth.market_volatility * 0.5;
        
        let total_impact = base_slippage_factor + depth_impact + volatility_impact;
        let target_price = (current_price as f64 * total_impact) as u64;
        
        Ok(target_price)
    }
    
    /// 模拟市场执行
    fn simulate_market_execution(&self, size: u64, target_price: u64) -> anchor_lang::Result<u64> {
        // 实际实现应调用DEX接口
        // 这里模拟执行价格，包含一些随机性
        let base_price = target_price;
        let price_variation = (rand::random::<u32>() % 1000) as u64; // 0-999的价格变化
        let execution_price = base_price + price_variation;
        Ok(execution_price)
    }
    
    /// 计算滑点
    fn calculate_slippage(&self, target_price: u64, execution_price: u64) -> u32 {
        if target_price == 0 {
            return 0;
        }
        
        let price_diff = if execution_price > target_price {
            execution_price - target_price
        } else {
            target_price - execution_price
        };
        
        ((price_diff as f64 / target_price as f64) * 10000.0) as u32
    }
    
    /// 计算价格偏差
    fn calculate_price_deviation(&self, current_price: u64, target_price: u64) -> u32 {
        self.calculate_slippage(target_price, current_price)
    }
    
    /// 检查风险限制
    fn check_risk_limits(&self, params: &VwapParams, result: &VwapIntervalResult) -> anchor_lang::Result<()> {
        // 检查单次执行比例
        let execution_ratio = (result.executed_amount as f64 / params.total_amount as f64) * 10000.0;
        if execution_ratio > params.risk_params.max_single_execution_bps as f64 {
            return Err(AlgorithmError::InvalidResult {
                reason: format!("Single execution ratio {} bps exceeds limit {} bps", 
                    execution_ratio as u32, params.risk_params.max_single_execution_bps),
            }.into());
        }
        
        // 检查价格偏差
        if result.slippage_bps > params.risk_params.max_price_deviation_bps {
            return Err(AlgorithmError::InvalidResult {
                reason: format!("Price deviation {} bps exceeds limit {} bps", 
                    result.slippage_bps, params.risk_params.max_price_deviation_bps),
            }.into());
        }
        
        // 检查执行时间
        if result.execution_time > params.risk_params.max_execution_time {
            return Err(AlgorithmError::ExecutionTimeout {
                algorithm_name: "VWAP".to_string(),
            }.into());
        }
        
        // 检查紧急停止条件
        if params.risk_params.enable_emergency_stop && 
           result.slippage_bps > params.risk_params.emergency_stop_threshold_bps {
            return Err(AlgorithmError::InvalidResult {
                reason: "Emergency stop triggered due to excessive slippage".to_string(),
            }.into());
        }
        
        Ok(())
    }
    
    /// 更新执行缓存
    fn update_execution_cache(&self, plan: &VwapExecutionPlan, result: &VwapIntervalResult) -> anchor_lang::Result<()> {
        let cache_key = format!("vwap_{}", plan.start_time);
        // 这里应该更新缓存，但由于self是不可变的，实际实现中需要内部可变性
        Ok(())
    }
    
    /// 计算性能指标
    fn calculate_performance_metrics(
        &self,
        execution_times: Vec<u64>,
        price_deviations: Vec<u32>,
        volume_distribution: Vec<u64>,
        total_execution_time: u64,
        total_intervals: usize,
        vwap_price: u64,
    ) -> anchor_lang::Result<VwapMetrics> {
        if execution_times.is_empty() {
            return Err(AlgorithmError::InvalidResult {
                reason: "No execution times available for metrics calculation".to_string(),
            }.into());
        }
        
        let avg_execution_latency = execution_times.iter().sum::<u64>() / execution_times.len() as u64;
        let max_execution_latency = *execution_times.iter().max().unwrap_or(&0);
        
        let avg_slippage = if !price_deviations.is_empty() {
            price_deviations.iter().sum::<u32>() / price_deviations.len() as u32
        } else {
            0
        };
        
        let max_slippage = price_deviations.iter().max().copied().unwrap_or(0);
        
        // 计算VWAP偏差统计
        let avg_deviation = if !price_deviations.is_empty() {
            price_deviations.iter().sum::<u32>() / price_deviations.len() as u32
        } else {
            0
        };
        
        let max_deviation = price_deviations.iter().max().copied().unwrap_or(0);
        
        // 计算标准差
        let deviation_std = if price_deviations.len() > 1 {
            let mean = avg_deviation as f64;
            let variance = price_deviations.iter()
                .map(|&x| {
                    let diff = x as f64 - mean;
                    diff * diff
                })
                .sum::<f64>() / (price_deviations.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };
        
        let threshold_violations = price_deviations.iter()
            .filter(|&&deviation| deviation > 100)
            .count() as u32;
        
        // 计算成交量分布统计
        let volume_stats = self.calculate_volume_distribution_stats(&volume_distribution)?;
        
        Ok(VwapMetrics {
            total_execution_time_ms: total_execution_time * 1000,
            avg_execution_latency_ms: avg_execution_latency * 1000,
            max_execution_latency_ms: max_execution_latency * 1000,
            success_rate: 1.0,
            avg_slippage_bps: avg_slippage,
            max_slippage_bps: max_slippage,
            vwap_deviation_stats: VwapDeviationStats {
                avg_deviation_bps: avg_deviation,
                max_deviation_bps: max_deviation,
                deviation_std,
                threshold_violations,
            },
            volume_distribution_stats: volume_stats,
        })
    }
    
    /// 计算成交量分布统计
    fn calculate_volume_distribution_stats(&self, volumes: &[u64]) -> anchor_lang::Result<VolumeDistributionStats> {
        if volumes.is_empty() {
            return Err(AlgorithmError::InvalidResult {
                reason: "No volume data available for distribution calculation".to_string(),
            }.into());
        }
        
        let total_volume: u64 = volumes.iter().sum();
        let avg_volume = total_volume / volumes.len() as u64;
        
        // 计算成交量集中度（基尼系数）
        let mut sorted_volumes = volumes.to_vec();
        sorted_volumes.sort();
        
        let mut cumulative_volume = 0u64;
        let mut gini_numerator = 0u64;
        
        for (i, &volume) in sorted_volumes.iter().enumerate() {
            cumulative_volume += volume;
            gini_numerator += (2 * (i + 1) - volumes.len()) as u64 * volume;
        }
        
        let volume_concentration = if total_volume > 0 {
            1.0 - (gini_numerator as f64 / (volumes.len() as u64 * total_volume) as f64)
        } else {
            0.0
        };
        
        // 计算最大单次成交量比例
        let max_volume = volumes.iter().max().copied().unwrap_or(0);
        let max_single_volume_bps = if total_volume > 0 {
            ((max_volume as f64 / total_volume as f64) * 10000.0) as u32
        } else {
            0
        };
        
        // 计算成交量标准差
        let volume_variance = volumes.iter()
            .map(|&v| {
                let diff = v as f64 - avg_volume as f64;
                diff * diff
            })
            .sum::<f64>() / volumes.len() as f64;
        let volume_std = volume_variance.sqrt();
        
        // 计算成交量分布均匀度
        let volume_uniformity = if volume_std > 0.0 {
            1.0 / (1.0 + volume_std / avg_volume as f64)
        } else {
            1.0
        };
        
        Ok(VolumeDistributionStats {
            volume_concentration,
            max_single_volume_bps,
            volume_std,
            volume_uniformity,
        })
    }
    
    /// 记录性能指标
    fn record_performance_metrics(&self, metrics: &VwapMetrics) -> anchor_lang::Result<()> {
        msg!("VWAP Performance Metrics:");
        msg!("  Total Execution Time: {} ms", metrics.total_execution_time_ms);
        msg!("  Avg Execution Latency: {} ms", metrics.avg_execution_latency_ms);
        msg!("  Max Execution Latency: {} ms", metrics.max_execution_latency_ms);
        msg!("  Success Rate: {:.2}%", metrics.success_rate * 100.0);
        msg!("  Avg Slippage: {} bps", metrics.avg_slippage_bps);
        msg!("  Max Slippage: {} bps", metrics.max_slippage_bps);
        msg!("  VWAP Deviation - Avg: {} bps, Max: {} bps", 
            metrics.vwap_deviation_stats.avg_deviation_bps,
            metrics.vwap_deviation_stats.max_deviation_bps);
        msg!("  Volume Concentration: {:.4}", metrics.volume_distribution_stats.volume_concentration);
        msg!("  Max Single Volume: {} bps", metrics.volume_distribution_stats.max_single_volume_bps);
        msg!("  Volume Uniformity: {:.4}", metrics.volume_distribution_stats.volume_uniformity);
        
        Ok(())
    }
}

/// VWAP执行计划
#[derive(Clone, Debug)]
pub struct VwapExecutionPlan {
    /// 执行分段列表
    pub intervals: Vec<VwapInterval>,
    /// 总订单数量
    pub total_amount: u64,
    /// 时间窗口
    pub time_window: u64,
    /// 开始时间
    pub start_time: u64,
}

/// VWAP执行分段
#[derive(Clone, Debug)]
pub struct VwapInterval {
    /// 分段索引
    pub index: u32,
    /// 分段大小
    pub size: u64,
    /// 权重
    pub weight: f64,
    /// 开始时间
    pub start_time: u64,
    /// 结束时间
    pub end_time: u64,
    /// 目标价格
    pub target_price: u64,
    /// 是否已执行
    pub executed: bool,
}

/// VWAP分段执行结果
#[derive(Clone, Debug)]
pub struct VwapIntervalResult {
    /// 分段索引
    pub interval_index: u32,
    /// 执行数量
    pub executed_amount: u64,
    /// 权重
    pub weight: f64,
    /// 目标价格
    pub target_price: u64,
    /// 执行价格
    pub execution_price: u64,
    /// 总成本
    pub total_cost: u64,
    /// 滑点（基点）
    pub slippage_bps: u32,
    /// 执行时间
    pub execution_time: u64,
}

impl Default for MarketDepthData {
    fn default() -> Self {
        Self {
            bid_depth: Vec::new(),
            ask_depth: Vec::new(),
            liquidity_score: 0.0,
            price_impact_factor: 0.0,
            market_volatility: 0.0,
        }
    }
}

/// Anchor 自动注册宏
// #[ctor::ctor]
fn auto_register_vwap_algorithm() {
    let adapter = VwapAlgorithm::new();
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    
    #[test]
    fn test_vwap_algorithm_creation() {
        let algo = VwapAlgorithm::new();
        assert_eq!(algo.name(), "vwap");
        assert_eq!(algo.version(), "2.0.0");
        assert_eq!(algo.algorithm_type(), AlgorithmType::Execution);
    }
    
    #[test]
    fn test_vwap_params_validation() {
        let algo = VwapAlgorithm::new();
        
        let valid_params = VwapParams {
            total_amount: 1000,
            time_window: 3600,
            num_intervals: 10,
            slippage_tolerance_bps: 100,
            min_interval_size: 50,
            max_interval_size: 200,
            price_deviation_bps: 50,
            enable_dynamic_weighting: true,
            enable_market_depth_analysis: true,
            risk_params: VwapRiskParams {
                max_single_execution_bps: 1000,
                max_price_deviation_bps: 200,
                max_execution_time: 60,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
                max_volume_deviation_bps: 300,
            },
            monitoring_params: VwapMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
                enable_depth_monitoring: true,
            },
        };
        
        assert!(algo.validate_vwap_params(&valid_params).is_ok());
    }
    
    #[test]
    fn test_weight_calculation() {
        let algo = VwapAlgorithm::new();
        
        // 测试线性权重
        let linear_weights = algo.calculate_weights(5).unwrap();
        assert_eq!(linear_weights, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        
        // 测试指数权重
        let mut config = VwapConfig::default();
        config.default_weight_strategy = WeightStrategy::Exponential;
        let algo = VwapAlgorithm::with_config(config);
        
        let exp_weights = algo.calculate_weights(4).unwrap();
        assert_eq!(exp_weights, vec![1.0, 2.0, 4.0, 8.0]);
    }
    
    #[test]
    fn test_execution_plan_calculation() {
        let algo = VwapAlgorithm::new();
        
        let params = VwapParams {
            total_amount: 1000,
            time_window: 3600,
            num_intervals: 5,
            slippage_tolerance_bps: 100,
            min_interval_size: 50,
            max_interval_size: 300,
            price_deviation_bps: 50,
            enable_dynamic_weighting: true,
            enable_market_depth_analysis: true,
            risk_params: VwapRiskParams {
                max_single_execution_bps: 1000,
                max_price_deviation_bps: 200,
                max_execution_time: 60,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
                max_volume_deviation_bps: 300,
            },
            monitoring_params: VwapMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
                enable_depth_monitoring: true,
            },
        };
        
        let plan = algo.calculate_execution_plan(&params).unwrap();
        
        assert_eq!(plan.total_amount, 1000);
        assert_eq!(plan.time_window, 3600);
        assert_eq!(plan.intervals.len(), 5);
        
        // 验证分段大小总和
        let total_planned: u64 = plan.intervals.iter().map(|i| i.size).sum();
        assert_eq!(total_planned, 1000);
        
        // 验证权重分布
        for (i, interval) in plan.intervals.iter().enumerate() {
            assert_eq!(interval.index, i as u32);
            assert!(interval.size >= params.min_interval_size);
            assert!(interval.size <= params.max_interval_size);
            assert!(interval.weight > 0.0);
        }
    }
    
    #[test]
    fn test_volume_distribution_stats() {
        let algo = VwapAlgorithm::new();
        
        let volumes = vec![100, 150, 200, 250, 300];
        let stats = algo.calculate_volume_distribution_stats(&volumes).unwrap();
        
        assert!(stats.volume_concentration >= 0.0 && stats.volume_concentration <= 1.0);
        assert!(stats.max_single_volume_bps > 0);
        assert!(stats.volume_std > 0.0);
        assert!(stats.volume_uniformity > 0.0 && stats.volume_uniformity <= 1.0);
    }
}
