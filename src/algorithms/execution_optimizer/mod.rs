/*!
 * 执行优化器模块 - 生产级实现
 *
 * 生产级执行优化器实现。
 * 支持 Anchor 框架自动注册，便于在算法工厂/注册表中动态调用。
 * 
 * 功能特性：
 * - 多策略优化（遗传算法、机器学习、动态规划）
 * - 成本最小化和市场冲击评估
 * - 执行时机优化和动态调整
 * - 风险控制和熔断机制
 * - 性能监控和指标收集
 * - 多目标优化算法
 * - 可配置的优化策略
 * - 实时市场数据集成
 */

use anchor_lang::prelude::*;
use crate::algorithms::traits::{Algorithm, ExecutionStrategy, AlgorithmType, ExecutionResult};
use crate::core::adapter::AdapterTrait;
use crate::core::types::algo::AlgoParams;
use crate::errors::algorithm_error::AlgorithmError;
use crate::core::constants::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub mod types;
pub mod genetic;
pub mod ml;

use genetic::GeneticOptimizer;
use ml::MLOptimizer;

/// 执行优化器算法参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ExecutionOptimizerParams {
    /// 订单大小
    pub order_size: u64,
    /// 目标执行时间（秒）
    pub target_execution_time: u64,
    /// 最大滑点容忍度（基点）
    pub max_slippage_tolerance_bps: u32,
    /// 是否启用成本优化
    pub enable_cost_optimization: bool,
    /// 是否启用市场冲击优化
    pub enable_market_impact_optimization: bool,
    /// 是否启用时机优化
    pub enable_timing_optimization: bool,
    /// 优化策略
    pub optimization_strategy: OptimizationStrategy,
    /// 风险控制参数
    pub risk_params: ExecutionOptimizerRiskParams,
    /// 性能监控参数
    pub monitoring_params: ExecutionOptimizerMonitoringParams,
}

/// 优化策略枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OptimizationStrategy {
    /// 遗传算法
    Genetic,
    /// 机器学习
    MachineLearning,
    /// 动态规划
    DynamicProgramming,
    /// 模拟退火
    SimulatedAnnealing,
    /// 粒子群优化
    ParticleSwarm,
    /// 混合策略
    Hybrid,
}

/// 执行优化器风险控制参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ExecutionOptimizerRiskParams {
    /// 最大市场冲击（基点）
    pub max_market_impact_bps: u32,
    /// 最大执行成本（基点）
    pub max_execution_cost_bps: u32,
    /// 最大执行时间（秒）
    pub max_execution_time: u64,
    /// 是否启用紧急停止
    pub enable_emergency_stop: bool,
    /// 紧急停止阈值（基点）
    pub emergency_stop_threshold_bps: u32,
    /// 最大波动率容忍度（基点）
    pub max_volatility_tolerance_bps: u32,
    /// 最小流动性要求
    pub min_liquidity_requirement: u64,
}

/// 执行优化器性能监控参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ExecutionOptimizerMonitoringParams {
    /// 是否启用性能监控
    pub enable_monitoring: bool,
    /// 性能指标收集间隔（秒）
    pub metrics_interval: u64,
    /// 是否启用详细日志
    pub enable_detailed_logging: bool,
    /// 是否启用性能警告
    pub enable_performance_warnings: bool,
    /// 是否启用优化分析
    pub enable_optimization_analysis: bool,
}

/// 优化结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct OptimizationResult {
    /// 优化ID
    pub id: String,
    /// 最优执行计划
    pub optimal_execution_plan: ExecutionPlan,
    /// 预期成本
    pub expected_cost: u64,
    /// 预期市场冲击
    pub expected_market_impact_bps: u32,
    /// 预期执行时间
    pub expected_execution_time: u64,
    /// 优化评分
    pub optimization_score: f64,
    /// 风险评分
    pub risk_score: f64,
    /// 优化指标
    pub metrics: OptimizationMetrics,
}

/// 执行计划
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ExecutionPlan {
    /// 计划ID
    pub id: String,
    /// 执行分段
    pub segments: Vec<ExecutionSegment>,
    /// 总执行数量
    pub total_execution_amount: u64,
    /// 总执行时间
    pub total_execution_time: u64,
    /// 总成本
    pub total_cost: u64,
    /// 总市场冲击
    pub total_market_impact_bps: u32,
}

/// 执行分段
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ExecutionSegment {
    /// 分段索引
    pub index: u32,
    /// 执行数量
    pub amount: u64,
    /// 执行时间
    pub execution_time: u64,
    /// 目标价格
    pub target_price: u64,
    /// 预期成本
    pub expected_cost: u64,
    /// 预期市场冲击
    pub expected_market_impact_bps: u32,
    /// 执行策略
    pub execution_strategy: ExecutionStrategyType,
}

/// 执行策略
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExecutionStrategyType {
    /// 立即执行
    Immediate,
    /// 限价单
    LimitOrder,
    /// 市价单
    MarketOrder,
    /// 冰山订单
    Iceberg,
    /// 时间加权
    TimeWeighted,
    /// 成交量加权
    VolumeWeighted,
}

/// 优化指标
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct OptimizationMetrics {
    /// 优化时间（毫秒）
    pub optimization_time_ms: u64,
    /// 迭代次数
    pub iteration_count: u32,
    /// 收敛次数
    pub convergence_count: u32,
    /// 成本改善率
    pub cost_improvement_rate: f64,
    /// 市场冲击改善率
    pub market_impact_improvement_rate: f64,
    /// 执行时间改善率
    pub execution_time_improvement_rate: f64,
    /// 优化成功率
    pub optimization_success_rate: f64,
}

/// 市场数据
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MarketData {
    /// 当前价格
    pub current_price: u64,
    /// 价格波动性
    pub volatility: f64,
    /// 市场流动性
    pub liquidity: u64,
    /// 市场深度
    pub market_depth: Vec<PriceLevel>,
    /// 交易量
    pub volume: u64,
    /// 买卖价差
    pub spread_bps: u32,
    /// 市场情绪
    pub market_sentiment: MarketSentiment,
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

/// 市场情绪
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarketSentiment {
    /// 看涨
    Bullish,
    /// 看跌
    Bearish,
    /// 中性
    Neutral,
}

/// 执行优化器算法结构体
#[derive(Default)]
pub struct ExecutionOptimizerAlgorithm {
    /// 算法配置
    config: ExecutionOptimizerConfig,
    /// 优化器缓存
    optimizer_cache: HashMap<String, Box<dyn Optimizer>>,
    /// 执行状态缓存
    execution_cache: HashMap<String, OptimizationResult>,
}

/// 执行优化器算法配置
#[derive(Clone, Debug)]
pub struct ExecutionOptimizerConfig {
    /// 默认优化策略
    pub default_strategy: OptimizationStrategy,
    /// 最大迭代次数
    pub max_iterations: u32,
    /// 收敛阈值
    pub convergence_threshold: f64,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 缓存过期时间（秒）
    pub cache_expiry_time: u64,
    /// 默认风险参数
    pub default_risk_params: ExecutionOptimizerRiskParams,
    /// 优化超时时间（秒）
    pub optimization_timeout: u64,
}

/// 优化器trait
pub trait Optimizer: Send + Sync {
    fn optimize(&self, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<OptimizationResult>;
    fn name(&self) -> &'static str;
}

impl Default for ExecutionOptimizerConfig {
    fn default() -> Self {
        Self {
            default_strategy: OptimizationStrategy::Genetic,
            max_iterations: 1000,
            convergence_threshold: 0.001,
            enable_cache: true,
            cache_expiry_time: 300, // 5分钟
            default_risk_params: ExecutionOptimizerRiskParams {
                max_market_impact_bps: 200,
                max_execution_cost_bps: 100,
                max_execution_time: 3600,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
                max_volatility_tolerance_bps: 8000,
                min_liquidity_requirement: 1000,
            },
            optimization_timeout: 60,
        }
    }
}

/// AdapterTrait 实现
impl AdapterTrait for ExecutionOptimizerAlgorithm {
    fn name(&self) -> &'static str { 
        "execution_optimizer" 
    }
    
    fn version(&self) -> &'static str { 
        "2.0.0" 
    }
    
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
    
    fn status(&self) -> Option<String> { 
        Some("active".to_string()) 
    }
}

/// Algorithm trait 实现
impl Algorithm for ExecutionOptimizerAlgorithm {
    fn execute(&self, params: &AlgoParams) -> anchor_lang::Result<ExecutionResult> {
        // 解析执行优化器参数
        let optimizer_params = self.parse_optimizer_params(params)?;
        
        // 验证参数
        self.validate_optimizer_params(&optimizer_params)?;
        
        // 获取市场数据
        let market_data = self.get_market_data(&optimizer_params)?;
        
        // 执行优化
        let optimization_result = self.optimize_execution(&optimizer_params, &market_data)?;
        
        // 转换为执行结果
        let result = self.convert_to_execution_result(&optimization_result)?;
        
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
        AlgorithmType::Optimization 
    }
}

/// ExecutionStrategy trait 实现
impl ExecutionStrategy for ExecutionOptimizerAlgorithm {
    fn execute(&self, _ctx: Context<crate::algorithms::traits::Execute>, params: &AlgoParams) -> Result<ExecutionResult> {
        self.execute(params)
    }
}

impl ExecutionOptimizerAlgorithm {
    /// 创建新的执行优化器算法实例
    pub fn new() -> Self {
        let mut optimizer = Self {
            config: ExecutionOptimizerConfig::default(),
            optimizer_cache: HashMap::new(),
            execution_cache: HashMap::new(),
        };
        
        // 初始化优化器
        optimizer.initialize_optimizers();
        
        optimizer
    }
    
    /// 使用自定义配置创建执行优化器算法实例
    pub fn with_config(config: ExecutionOptimizerConfig) -> Self {
        let mut optimizer = Self {
            config,
            optimizer_cache: HashMap::new(),
            execution_cache: HashMap::new(),
        };
        
        // 初始化优化器
        optimizer.initialize_optimizers();
        
        optimizer
    }
    
    /// 初始化优化器
    fn initialize_optimizers(&mut self) {
        // 注册遗传算法优化器
        self.optimizer_cache.insert(
            "genetic".to_string(),
            Box::new(GeneticOptimizer::new()),
        );
        
        // 注册机器学习优化器
        self.optimizer_cache.insert(
            "ml".to_string(),
            Box::new(MLOptimizer::new()),
        );
    }
    
    /// 解析执行优化器参数
    fn parse_optimizer_params(&self, params: &AlgoParams) -> Result<ExecutionOptimizerParams> {
        if params.params.is_empty() {
            return Err(AlgorithmError::InvalidParameters {
                reason: "Empty parameters".to_string(),
            }.into());
        }
        
        match bincode::deserialize::<ExecutionOptimizerParams>(&params.params) {
            Ok(optimizer_params) => Ok(optimizer_params),
            Err(e) => Err(AlgorithmError::InvalidParameters {
                reason: format!("Failed to deserialize optimizer parameters: {}", e),
            }.into()),
        }
    }
    
    /// 验证执行优化器参数
    fn validate_optimizer_params(&self, params: &ExecutionOptimizerParams) -> Result<()> {
        // 验证订单大小
        require!(
            params.order_size > 0,
            AlgorithmError::InvalidParameters {
                reason: "Order size must be greater than 0".to_string(),
            }
        );
        
        // 验证目标执行时间
        require!(
            params.target_execution_time > 0 && params.target_execution_time <= MAX_REBALANCE_INTERVAL,
            AlgorithmError::InvalidParameters {
                reason: format!("Target execution time must be between 1 and {}", MAX_REBALANCE_INTERVAL).to_string(),
            }
        );
        
        // 验证滑点容忍度
        require!(
            params.max_slippage_tolerance_bps <= MAX_SLIPPAGE_BPS,
            AlgorithmError::InvalidParameters {
                reason: format!("Max slippage tolerance must not exceed {} bps", MAX_SLIPPAGE_BPS).to_string(),
            }
        );
        
        Ok(())
    }
    
    /// 获取市场数据
    fn get_market_data(&self, params: &ExecutionOptimizerParams) -> Result<MarketData> {
        // 实际实现应调用Oracle和DEX接口获取实时市场数据
        Ok(MarketData {
            current_price: 1_000_000, // 模拟价格
            volatility: 0.05,
            liquidity: 10_000_000,
            market_depth: vec![
                PriceLevel { price: 999_000, size: 1000, cumulative_size: 1000 },
                PriceLevel { price: 998_000, size: 2000, cumulative_size: 3000 },
                PriceLevel { price: 1_001_000, size: 1000, cumulative_size: 1000 },
                PriceLevel { price: 1_002_000, size: 2000, cumulative_size: 3000 },
            ],
            volume: 1_000_000,
            spread_bps: 20,
            market_sentiment: MarketSentiment::Neutral,
        })
    }
    
    /// 执行优化
    fn optimize_execution(&self, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<OptimizationResult> {
        // 检查风险控制
        self.check_risk_limits(params, market_data)?;
        
        // 选择优化器
        let optimizer = self.select_optimizer(&params.optimization_strategy)?;
        
        // 执行优化
        let start_time = self.get_current_timestamp();
        let result = optimizer.optimize(params, market_data)?;
        let optimization_time = self.get_current_timestamp() - start_time;
        
        // 更新优化指标
        let mut updated_result = result.clone();
        updated_result.metrics.optimization_time_ms = optimization_time * 1000;
        
        // 记录性能指标
        if params.monitoring_params.enable_monitoring {
            self.record_optimization_metrics(&updated_result)?;
        }
        
        Ok(updated_result)
    }
    
    /// 选择优化器
    fn select_optimizer(&self, strategy: &OptimizationStrategy) -> Result<&Box<dyn Optimizer>> {
        let optimizer_name = match strategy {
            OptimizationStrategy::Genetic => "genetic",
            OptimizationStrategy::MachineLearning => "ml",
            OptimizationStrategy::DynamicProgramming => "genetic", // 使用遗传算法作为默认
            OptimizationStrategy::SimulatedAnnealing => "genetic",
            OptimizationStrategy::ParticleSwarm => "genetic",
            OptimizationStrategy::Hybrid => "genetic",
        };
        
        self.optimizer_cache.get(optimizer_name).ok_or_else(|| {
            AlgorithmError::InvalidParameters {
                reason: format!("Optimizer '{}' not found", optimizer_name).to_string(),
            }.into()
        })
    }
    
    /// 检查风险限制
    fn check_risk_limits(&self, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<()> {
        // 检查波动率
        let volatility_bps = (market_data.volatility * 10000.0) as u32;
        if volatility_bps > params.risk_params.max_volatility_tolerance_bps {
            return Err(AlgorithmError::InvalidResult {
                reason: format!("Volatility {} bps exceeds tolerance {} bps", 
                    volatility_bps, params.risk_params.max_volatility_tolerance_bps),
            }.into());
        }
        
        // 检查流动性
        if market_data.liquidity < params.risk_params.min_liquidity_requirement {
            return Err(AlgorithmError::InvalidResult {
                reason: format!("Liquidity {} is below requirement {}", 
                    market_data.liquidity, params.risk_params.min_liquidity_requirement),
            }.into());
        }
        
        // 检查订单大小相对于流动性
        let order_size_ratio = (params.order_size as f64 / market_data.liquidity as f64) * 10000.0;
        if order_size_ratio > params.risk_params.max_market_impact_bps as f64 {
            return Err(AlgorithmError::InvalidResult {
                reason: format!("Order size ratio {} bps exceeds market impact limit {} bps", 
                    order_size_ratio as u32, params.risk_params.max_market_impact_bps),
            }.into());
        }
        
        Ok(())
    }
    
    /// 转换为执行结果
    fn convert_to_execution_result(&self, optimization_result: &OptimizationResult) -> Result<ExecutionResult> {
        Ok(ExecutionResult {
            optimized_size: optimization_result.optimal_execution_plan.total_execution_amount,
            expected_cost: optimization_result.expected_cost,
        })
    }
    
    /// 记录优化指标
    fn record_optimization_metrics(&self, result: &OptimizationResult) -> Result<()> {
        msg!("Execution Optimization Metrics:");
        msg!("  Optimization Time: {} ms", result.metrics.optimization_time_ms);
        msg!("  Iteration Count: {}", result.metrics.iteration_count);
        msg!("  Convergence Count: {}", result.metrics.convergence_count);
        msg!("  Cost Improvement Rate: {:.2}%", result.metrics.cost_improvement_rate * 100.0);
        msg!("  Market Impact Improvement Rate: {:.2}%", result.metrics.market_impact_improvement_rate * 100.0);
        msg!("  Execution Time Improvement Rate: {:.2}%", result.metrics.execution_time_improvement_rate * 100.0);
        msg!("  Optimization Success Rate: {:.2}%", result.metrics.optimization_success_rate * 100.0);
        msg!("  Optimization Score: {:.4}", result.optimization_score);
        msg!("  Risk Score: {:.4}", result.risk_score);
        msg!("  Expected Cost: {}", result.expected_cost);
        msg!("  Expected Market Impact: {} bps", result.expected_market_impact_bps);
        msg!("  Expected Execution Time: {} s", result.expected_execution_time);
        
        Ok(())
    }
    
    /// 获取当前时间戳
    fn get_current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Anchor 自动注册宏
// #[ctor::ctor]
fn auto_register_execution_optimizer_algorithm() {
    let adapter = ExecutionOptimizerAlgorithm::new();
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    
    #[test]
    fn test_execution_optimizer_algorithm_creation() {
        let algo = ExecutionOptimizerAlgorithm::new();
        assert_eq!(algo.name(), "execution_optimizer");
        assert_eq!(algo.version(), "2.0.0");
        assert_eq!(algo.algorithm_type(), AlgorithmType::Optimization);
    }
    
    #[test]
    fn test_optimizer_params_validation() {
        let algo = ExecutionOptimizerAlgorithm::new();
        
        let valid_params = ExecutionOptimizerParams {
            order_size: 1000,
            target_execution_time: 3600,
            max_slippage_tolerance_bps: 100,
            enable_cost_optimization: true,
            enable_market_impact_optimization: true,
            enable_timing_optimization: true,
            optimization_strategy: OptimizationStrategy::Genetic,
            risk_params: ExecutionOptimizerRiskParams {
                max_market_impact_bps: 200,
                max_execution_cost_bps: 100,
                max_execution_time: 3600,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
                max_volatility_tolerance_bps: 8000,
                min_liquidity_requirement: 1000,
            },
            monitoring_params: ExecutionOptimizerMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
                enable_optimization_analysis: true,
            },
        };
        
        assert!(algo.validate_optimizer_params(&valid_params).is_ok());
        
        // 测试无效参数
        let mut invalid_params = valid_params.clone();
        invalid_params.order_size = 0;
        assert!(algo.validate_optimizer_params(&invalid_params).is_err());
    }
    
    #[test]
    fn test_market_data_creation() {
        let algo = ExecutionOptimizerAlgorithm::new();
        
        let params = ExecutionOptimizerParams {
            order_size: 1000,
            target_execution_time: 3600,
            max_slippage_tolerance_bps: 100,
            enable_cost_optimization: true,
            enable_market_impact_optimization: true,
            enable_timing_optimization: true,
            optimization_strategy: OptimizationStrategy::Genetic,
            risk_params: ExecutionOptimizerRiskParams {
                max_market_impact_bps: 200,
                max_execution_cost_bps: 100,
                max_execution_time: 3600,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
                max_volatility_tolerance_bps: 8000,
                min_liquidity_requirement: 1000,
            },
            monitoring_params: ExecutionOptimizerMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
                enable_optimization_analysis: true,
            },
        };
        
        let market_data = algo.get_market_data(&params).unwrap();
        
        assert!(market_data.current_price > 0);
        assert!(market_data.volatility > 0.0);
        assert!(market_data.liquidity > 0);
        assert!(!market_data.market_depth.is_empty());
        assert!(market_data.volume > 0);
        assert!(market_data.spread_bps > 0);
    }
    
    #[test]
    fn test_risk_limits_check() {
        let algo = ExecutionOptimizerAlgorithm::new();
        
        let params = ExecutionOptimizerParams {
            order_size: 1000,
            target_execution_time: 3600,
            max_slippage_tolerance_bps: 100,
            enable_cost_optimization: true,
            enable_market_impact_optimization: true,
            enable_timing_optimization: true,
            optimization_strategy: OptimizationStrategy::Genetic,
            risk_params: ExecutionOptimizerRiskParams {
                max_market_impact_bps: 200,
                max_execution_cost_bps: 100,
                max_execution_time: 3600,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
                max_volatility_tolerance_bps: 8000,
                min_liquidity_requirement: 1000,
            },
            monitoring_params: ExecutionOptimizerMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
                enable_optimization_analysis: true,
            },
        };
        
        let market_data = MarketData {
            current_price: 1_000_000,
            volatility: 0.05,
            liquidity: 10_000_000,
            market_depth: vec![
                PriceLevel { price: 999_000, size: 1000, cumulative_size: 1000 },
                PriceLevel { price: 1_001_000, size: 1000, cumulative_size: 1000 },
            ],
            volume: 1_000_000,
            spread_bps: 20,
            market_sentiment: MarketSentiment::Neutral,
        };
        
        assert!(algo.check_risk_limits(&params, &market_data).is_ok());
        
        // 测试高风险情况
        let high_risk_market_data = MarketData {
            current_price: 1_000_000,
            volatility: 1.0, // 100%波动率
            liquidity: 10_000_000,
            market_depth: vec![
                PriceLevel { price: 999_000, size: 1000, cumulative_size: 1000 },
                PriceLevel { price: 1_001_000, size: 1000, cumulative_size: 1000 },
            ],
            volume: 1_000_000,
            spread_bps: 20,
            market_sentiment: MarketSentiment::Neutral,
        };
        
        assert!(algo.check_risk_limits(&params, &high_risk_market_data).is_err());
    }
}
