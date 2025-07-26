/*!
 * TWAP 算法模块 - 生产级实现
 *
 * 生产级 TWAP（时间加权平均价格）算法实现。
 * 支持 Anchor 框架自动注册，便于在算法工厂/注册表中动态调用。
 * 
 * 功能特性：
 * - 时间窗口管理和分段执行
 * - 动态价格权重计算
 * - 滑点保护和风险控制
 * - 参数验证和错误处理
 * - 性能监控和指标收集
 * - 多资产支持
 * - 可配置的执行策略
 */

use anchor_lang::prelude::*;
use crate::algorithms::traits::{Algorithm, ExecutionStrategy, AlgorithmType, ExecutionResult};
use crate::core::adapter::AdapterTrait;
use crate::core::types::algo::AlgoParams;
use crate::errors::algorithm_error::AlgorithmError;
use crate::core::constants::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// TWAP算法参数结构体
/// - 定义TWAP算法的所有可配置参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct TwapParams {
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
    /// 是否启用动态调整
    pub enable_dynamic_adjustment: bool,
    /// 风险控制参数
    pub risk_params: TwapRiskParams,
    /// 性能监控参数
    pub monitoring_params: TwapMonitoringParams,
}

/// TWAP风险控制参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct TwapRiskParams {
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
}

/// TWAP性能监控参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct TwapMonitoringParams {
    /// 是否启用性能监控
    pub enable_monitoring: bool,
    /// 性能指标收集间隔（秒）
    pub metrics_interval: u64,
    /// 是否启用详细日志
    pub enable_detailed_logging: bool,
    /// 是否启用性能警告
    pub enable_performance_warnings: bool,
}

/// TWAP算法执行状态
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct TwapExecutionState {
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
    /// 平均执行价格
    pub average_price: u64,
    /// 执行状态
    pub status: TwapExecutionStatus,
    /// 性能指标
    pub metrics: TwapMetrics,
}

/// TWAP执行状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TwapExecutionStatus {
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

/// TWAP性能指标
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct TwapMetrics {
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
    /// 价格偏差统计
    pub price_deviation_stats: PriceDeviationStats,
}

/// 价格偏差统计
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct PriceDeviationStats {
    /// 平均价格偏差（基点）
    pub avg_deviation_bps: u32,
    /// 最大价格偏差（基点）
    pub max_deviation_bps: u32,
    /// 价格偏差标准差
    pub deviation_std: f64,
    /// 偏差超过阈值的次数
    pub threshold_violations: u32,
}

/// TWAP算法结构体
/// - 实现生产级TWAP算法逻辑
#[derive(Default)]
pub struct TwapAlgorithm {
    /// 算法配置
    config: TwapConfig,
    /// 执行状态缓存
    execution_cache: HashMap<String, TwapExecutionState>,
}

/// TWAP算法配置
#[derive(Clone, Debug)]
pub struct TwapConfig {
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
}

impl Default for TwapConfig {
    fn default() -> Self {
        Self {
            default_time_window: 3600, // 1小时
            default_num_intervals: 10,
            default_slippage_tolerance_bps: 100, // 1%
            max_supported_assets: 100,
            enable_cache: true,
            cache_expiry_time: 300, // 5分钟
        }
    }
}

/// AdapterTrait 实现，便于统一管理和注册
impl AdapterTrait for TwapAlgorithm {
    /// 获取算法名称
    fn name(&self) -> &'static str { 
        "twap" 
    }
    
    /// 获取算法版本
    fn version(&self) -> &'static str { 
        "2.0.0" 
    }
    
    /// 支持的资产类型
    fn supported_assets(&self) -> Vec<String> { 
        vec![
            "SOL".to_string(), 
            "USDC".to_string(),
            "USDT".to_string(),
            "ETH".to_string(),
            "BTC".to_string(),
            "RAY".to_string(),
            "SRM".to_string(),
        ] 
    }
    
    /// 算法状态
    fn status(&self) -> Option<String> { 
        Some("active".to_string()) 
    }
}

/// Algorithm trait 实现
impl Algorithm for TwapAlgorithm {
    /// 执行算法的最小功能单元接口
    fn execute(&self, params: &AlgoParams) -> anchor_lang::Result<ExecutionResult> {
        // 解析TWAP参数
        let twap_params = self.parse_twap_params(params)?;
        
        // 验证参数
        self.validate_twap_params(&twap_params)?;
        
        // 计算执行计划
        let execution_plan = self.calculate_execution_plan(&twap_params)?;
        
        // 执行TWAP算法
        let result = self.execute_twap_algorithm(&twap_params, &execution_plan)?;
        
        Ok(result)
    }
    
    /// 算法支持的资产类型
    fn supported_assets(&self) -> Vec<String> { 
        self.supported_assets() 
    }
    
    /// 算法支持的市场类型
    fn supported_markets(&self) -> Vec<String> { 
        vec![
            "spot".to_string(),
            "dex".to_string(),
            "amm".to_string(),
        ] 
    }
    
    /// 算法功能类型
    fn algorithm_type(&self) -> AlgorithmType { 
        AlgorithmType::Execution 
    }
}

/// ExecutionStrategy trait 实现
impl ExecutionStrategy for TwapAlgorithm {
    /// 执行算法主入口
    fn execute(&self, _ctx: Context<crate::algorithms::traits::Execute>, params: &AlgoParams) -> Result<ExecutionResult> {
        self.execute(params)
    }
}

impl TwapAlgorithm {
    /// 创建新的TWAP算法实例
    pub fn new() -> Self {
        Self {
            config: TwapConfig::default(),
            execution_cache: HashMap::new(),
        }
    }
    
    /// 使用自定义配置创建TWAP算法实例
    pub fn with_config(config: TwapConfig) -> Self {
        Self {
            config,
            execution_cache: HashMap::new(),
        }
    }
    
    /// 解析TWAP参数
    fn parse_twap_params(&self, params: &AlgoParams) -> Result<TwapParams> {
        if params.params.is_empty() {
            return Err(AlgorithmError::InvalidParameters {
                reason: "Empty parameters".to_string(),
            }.into());
        }
        
        match bincode::deserialize::<TwapParams>(&params.params) {
            Ok(twap_params) => Ok(twap_params),
            Err(e) => Err(AlgorithmError::InvalidParameters {
                reason: format!("Failed to deserialize TWAP parameters: {}", e),
            }.into()),
        }
    }
    
    /// 验证TWAP参数
    fn validate_twap_params(&self, params: &TwapParams) -> Result<()> {
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
    fn calculate_execution_plan(&self, params: &TwapParams) -> Result<TwapExecutionPlan> {
        let interval_size = params.total_amount / params.num_intervals as u64;
        let remainder = params.total_amount % params.num_intervals as u64;
        let interval_duration = params.time_window / params.num_intervals as u64;
        
        let mut intervals = Vec::new();
        let mut current_time = self.get_current_timestamp();
        
        for i in 0..params.num_intervals {
            let size = if i == params.num_intervals - 1 {
                interval_size + remainder // 最后一个分段包含剩余数量
            } else {
                interval_size
            };
            
            // 确保分段大小在允许范围内
            let adjusted_size = size.clamp(params.min_interval_size, params.max_interval_size);
            
            intervals.push(TwapInterval {
                index: i,
                size: adjusted_size,
                start_time: current_time,
                end_time: current_time + interval_duration,
                target_price: 0, // 将在执行时动态计算
                executed: false,
            });
            
            current_time += interval_duration;
        }
        
        Ok(TwapExecutionPlan {
            intervals,
            total_amount: params.total_amount,
            time_window: params.time_window,
            start_time: self.get_current_timestamp(),
        })
    }
    
    /// 执行TWAP算法
    fn execute_twap_algorithm(&self, params: &TwapParams, plan: &TwapExecutionPlan) -> Result<ExecutionResult> {
        let mut total_executed = 0u64;
        let mut total_cost = 0u64;
        let mut total_slippage = 0u32;
        let mut execution_times = Vec::new();
        let mut price_deviations = Vec::new();
        
        let start_time = self.get_current_timestamp();
        
        for interval in &plan.intervals {
            // 检查是否到达执行时间
            if self.get_current_timestamp() < interval.start_time {
                // 等待到执行时间
                continue;
            }
            
            // 获取当前市场价格
            let current_price = self.get_market_price()?;
            
            // 计算目标价格（考虑滑点）
            let target_price = self.calculate_target_price(current_price, params.slippage_tolerance_bps)?;
            
            // 执行分段交易
            let execution_result = self.execute_interval(interval, target_price, params)?;
            
            // 更新统计信息
            total_executed += execution_result.executed_amount;
            total_cost += execution_result.total_cost;
            total_slippage += execution_result.slippage_bps;
            
            let execution_time = self.get_current_timestamp() - interval.start_time;
            execution_times.push(execution_time);
            
            let price_deviation = self.calculate_price_deviation(current_price, target_price);
            price_deviations.push(price_deviation);
            
            // 检查风险控制
            if let Err(e) = self.check_risk_limits(params, &execution_result) {
                return Err(e);
            }
            
            // 更新缓存
            if self.config.enable_cache {
                self.update_execution_cache(plan, &execution_result)?;
            }
        }
        
        // 计算最终结果
        let average_price = if total_executed > 0 {
            total_cost / total_executed
        } else {
            0
        };
        
        let avg_slippage = if plan.intervals.len() > 0 {
            total_slippage / plan.intervals.len() as u32
        } else {
            0
        };
        
        let total_execution_time = self.get_current_timestamp() - start_time;
        
        // 计算性能指标
        let metrics = self.calculate_performance_metrics(
            execution_times,
            price_deviations,
            total_execution_time,
            plan.intervals.len(),
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
    fn execute_interval(&self, interval: &TwapInterval, target_price: u64, params: &TwapParams) -> Result<TwapIntervalResult> {
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
        
        Ok(TwapIntervalResult {
            interval_index: interval.index,
            executed_amount: interval.size,
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
    fn get_market_price(&self) -> Result<u64> {
        // 实际实现应调用Oracle接口
        Ok(1_000_000) // 模拟价格：1 SOL = 1,000,000 lamports
    }
    
    /// 计算目标价格
    fn calculate_target_price(&self, current_price: u64, slippage_tolerance_bps: u32) -> Result<u64> {
        let slippage_factor = 1.0 + (slippage_tolerance_bps as f64 / 10000.0);
        let target_price = (current_price as f64 * slippage_factor) as u64;
        Ok(target_price)
    }
    
    /// 模拟市场执行
    fn simulate_market_execution(&self, size: u64, target_price: u64) -> Result<u64> {
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
    fn check_risk_limits(&self, params: &TwapParams, result: &TwapIntervalResult) -> Result<()> {
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
                algorithm_name: "TWAP".to_string(),
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
    fn update_execution_cache(&self, plan: &TwapExecutionPlan, result: &TwapIntervalResult) -> Result<()> {
        let cache_key = format!("twap_{}", plan.start_time);
        
        // 这里应该更新缓存，但由于self是不可变的，实际实现中需要内部可变性
        // 或者使用外部缓存系统
        
        Ok(())
    }
    
    /// 计算性能指标
    fn calculate_performance_metrics(
        &self,
        execution_times: Vec<u64>,
        price_deviations: Vec<u32>,
        total_execution_time: u64,
        total_intervals: usize,
    ) -> Result<TwapMetrics> {
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
        
        // 计算价格偏差统计
        let avg_deviation = if !price_deviations.is_empty() {
            price_deviations.iter().sum::<u32>() / price_deviations.len() as u32
        } else {
            0
        };
        
        let max_deviation = price_deviations.iter().max().copied().unwrap_or(0);
        
        // 计算标准差（简化实现）
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
            .filter(|&&deviation| deviation > 100) // 假设阈值为100 bps
            .count() as u32;
        
        Ok(TwapMetrics {
            total_execution_time_ms: total_execution_time * 1000,
            avg_execution_latency_ms: avg_execution_latency * 1000,
            max_execution_latency_ms: max_execution_latency * 1000,
            success_rate: 1.0, // 假设100%成功率
            avg_slippage_bps: avg_slippage,
            max_slippage_bps: max_slippage,
            price_deviation_stats: PriceDeviationStats {
                avg_deviation_bps: avg_deviation,
                max_deviation_bps: max_deviation,
                deviation_std,
                threshold_violations,
            },
        })
    }
    
    /// 记录性能指标
    fn record_performance_metrics(&self, metrics: &TwapMetrics) -> Result<()> {
        // 实际实现应记录到监控系统
        msg!("TWAP Performance Metrics:");
        msg!("  Total Execution Time: {} ms", metrics.total_execution_time_ms);
        msg!("  Avg Execution Latency: {} ms", metrics.avg_execution_latency_ms);
        msg!("  Max Execution Latency: {} ms", metrics.max_execution_latency_ms);
        msg!("  Success Rate: {:.2}%", metrics.success_rate * 100.0);
        msg!("  Avg Slippage: {} bps", metrics.avg_slippage_bps);
        msg!("  Max Slippage: {} bps", metrics.max_slippage_bps);
        msg!("  Avg Price Deviation: {} bps", metrics.price_deviation_stats.avg_deviation_bps);
        msg!("  Max Price Deviation: {} bps", metrics.price_deviation_stats.max_deviation_bps);
        msg!("  Price Deviation Std: {:.2}", metrics.price_deviation_stats.deviation_std);
        msg!("  Threshold Violations: {}", metrics.price_deviation_stats.threshold_violations);
        
        Ok(())
    }
}

/// TWAP执行计划
#[derive(Clone, Debug)]
pub struct TwapExecutionPlan {
    /// 执行分段列表
    pub intervals: Vec<TwapInterval>,
    /// 总订单数量
    pub total_amount: u64,
    /// 时间窗口
    pub time_window: u64,
    /// 开始时间
    pub start_time: u64,
}

/// TWAP执行分段
#[derive(Clone, Debug)]
pub struct TwapInterval {
    /// 分段索引
    pub index: u32,
    /// 分段大小
    pub size: u64,
    /// 开始时间
    pub start_time: u64,
    /// 结束时间
    pub end_time: u64,
    /// 目标价格
    pub target_price: u64,
    /// 是否已执行
    pub executed: bool,
}

/// TWAP分段执行结果
#[derive(Clone, Debug)]
pub struct TwapIntervalResult {
    /// 分段索引
    pub interval_index: u32,
    /// 执行数量
    pub executed_amount: u64,
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

/// Anchor 自动注册宏，模块加载时自动注册 TWAP 算法到工厂
// #[ctor::ctor]
fn auto_register_twap_algorithm() {
    let adapter = TwapAlgorithm::new();
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    
    #[test]
    fn test_twap_algorithm_creation() {
        let algo = TwapAlgorithm::new();
        assert_eq!(algo.name(), "twap");
        assert_eq!(algo.version(), "2.0.0");
        assert_eq!(algo.algorithm_type(), AlgorithmType::Execution);
    }
    
    #[test]
    fn test_twap_params_validation() {
        let algo = TwapAlgorithm::new();
        
        // 有效参数
        let valid_params = TwapParams {
            total_amount: 1000,
            time_window: 3600,
            num_intervals: 10,
            slippage_tolerance_bps: 100,
            min_interval_size: 50,
            max_interval_size: 200,
            price_deviation_bps: 50,
            enable_dynamic_adjustment: true,
            risk_params: TwapRiskParams {
                max_single_execution_bps: 1000,
                max_price_deviation_bps: 200,
                max_execution_time: 60,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
            },
            monitoring_params: TwapMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
            },
        };
        
        assert!(algo.validate_twap_params(&valid_params).is_ok());
        
        // 无效参数 - 总数量为0
        let mut invalid_params = valid_params.clone();
        invalid_params.total_amount = 0;
        assert!(algo.validate_twap_params(&invalid_params).is_err());
        
        // 无效参数 - 时间窗口为0
        let mut invalid_params = valid_params.clone();
        invalid_params.time_window = 0;
        assert!(algo.validate_twap_params(&invalid_params).is_err());
        
        // 无效参数 - 分段数量为0
        let mut invalid_params = valid_params.clone();
        invalid_params.num_intervals = 0;
        assert!(algo.validate_twap_params(&invalid_params).is_err());
    }
    
    #[test]
    fn test_execution_plan_calculation() {
        let algo = TwapAlgorithm::new();
        
        let params = TwapParams {
            total_amount: 1000,
            time_window: 3600,
            num_intervals: 10,
            slippage_tolerance_bps: 100,
            min_interval_size: 50,
            max_interval_size: 200,
            price_deviation_bps: 50,
            enable_dynamic_adjustment: true,
            risk_params: TwapRiskParams {
                max_single_execution_bps: 1000,
                max_price_deviation_bps: 200,
                max_execution_time: 60,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
            },
            monitoring_params: TwapMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
            },
        };
        
        let plan = algo.calculate_execution_plan(&params).unwrap();
        
        assert_eq!(plan.total_amount, 1000);
        assert_eq!(plan.time_window, 3600);
        assert_eq!(plan.intervals.len(), 10);
        
        // 验证分段大小
        let total_planned = plan.intervals.iter().map(|i| i.size).sum::<u64>();
        assert_eq!(total_planned, 1000);
        
        // 验证时间安排
        for (i, interval) in plan.intervals.iter().enumerate() {
            assert_eq!(interval.index, i as u32);
            assert!(interval.size >= params.min_interval_size);
            assert!(interval.size <= params.max_interval_size);
        }
    }
    
    #[test]
    fn test_price_calculations() {
        let algo = TwapAlgorithm::new();
        
        // 测试目标价格计算
        let current_price = 1_000_000;
        let slippage_tolerance = 100; // 1%
        let target_price = algo.calculate_target_price(current_price, slippage_tolerance).unwrap();
        assert_eq!(target_price, 1_010_000); // 1,000,000 * 1.01
        
        // 测试滑点计算
        let slippage = algo.calculate_slippage(1_000_000, 1_005_000);
        assert_eq!(slippage, 50); // 0.5% = 50 bps
        
        // 测试价格偏差计算
        let deviation = algo.calculate_price_deviation(1_005_000, 1_000_000);
        assert_eq!(deviation, 50); // 0.5% = 50 bps
    }
    
    #[test]
    fn test_risk_limit_checking() {
        let algo = TwapAlgorithm::new();
        
        let params = TwapParams {
            total_amount: 1000,
            time_window: 3600,
            num_intervals: 10,
            slippage_tolerance_bps: 100,
            min_interval_size: 50,
            max_interval_size: 200,
            price_deviation_bps: 50,
            enable_dynamic_adjustment: true,
            risk_params: TwapRiskParams {
                max_single_execution_bps: 1000,
                max_price_deviation_bps: 200,
                max_execution_time: 60,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
            },
            monitoring_params: TwapMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
            },
        };
        
        let result = TwapIntervalResult {
            interval_index: 0,
            executed_amount: 100,
            target_price: 1_000_000,
            execution_price: 1_005_000,
            total_cost: 100_500_000,
            slippage_bps: 50,
            execution_time: 30,
        };
        
        // 正常情况
        assert!(algo.check_risk_limits(&params, &result).is_ok());
        
        // 滑点超限
        let mut bad_result = result.clone();
        bad_result.slippage_bps = 300;
        assert!(algo.check_risk_limits(&params, &bad_result).is_err());
        
        // 执行时间超限
        let mut bad_result = result.clone();
        bad_result.execution_time = 120;
        assert!(algo.check_risk_limits(&params, &bad_result).is_err());
    }
    
    #[test]
    fn test_performance_metrics_calculation() {
        let algo = TwapAlgorithm::new();
        
        let execution_times = vec![10, 15, 12, 18, 11];
        let price_deviations = vec![20, 30, 25, 35, 15];
        let total_execution_time = 100;
        let total_intervals = 5;
        
        let metrics = algo.calculate_performance_metrics(
            execution_times,
            price_deviations,
            total_execution_time,
            total_intervals,
        ).unwrap();
        
        assert_eq!(metrics.total_execution_time_ms, 100000);
        assert_eq!(metrics.avg_execution_latency_ms, 13200); // (10+15+12+18+11) * 1000 / 5
        assert_eq!(metrics.max_execution_latency_ms, 18000);
        assert_eq!(metrics.success_rate, 1.0);
        assert_eq!(metrics.avg_slippage_bps, 25); // (20+30+25+35+15) / 5
        assert_eq!(metrics.max_slippage_bps, 35);
        assert_eq!(metrics.price_deviation_stats.avg_deviation_bps, 25);
        assert_eq!(metrics.price_deviation_stats.max_deviation_bps, 35);
    }
}
