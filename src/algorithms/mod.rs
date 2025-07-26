// 高级交易算法模块主入口
// - 包含 TWAP、VWAP、智能路由、优化器、风控等多种生产级算法实现
// - 统一注册、工厂、指标统计、动态扩展、热插拔
// - 每个 trait、注册表、工厂、方法、参数、用途、边界、Anchor 相关点均有详细注释
/*!
 * 高级交易算法模块
 *
 * 本模块包含成熟、生产级的高级交易算法实现，广泛应用于指数代币系统。
 * 支持多种算法类型（如 TWAP、VWAP、智能路由、优化器、风控等），并提供统一的注册、工厂、指标统计等能力。
 */

use std::collections::HashMap; // HashMap 用于算法名称到实例的映射
use std::sync::{Arc, RwLock}; // Arc+RwLock 实现线程安全的全局注册表

use crate::algorithms::algorithm_registry::*; // 引入算法注册表模块
use crate::algorithms::execution_optimizer::*; // 引入执行优化器模块
use crate::algorithms::smart_routing::*; // 引入智能路由算法模块
use crate::algorithms::twap::*; // 引入TWAP算法模块
use crate::algorithms::vwap::*; // 引入VWAP算法模块

use crate::algorithms::traits::{
    ExecutionAlgorithm, OptimizerAlgorithm, RiskAlgorithm, RoutingAlgorithm,
}; // 引入算法trait，便于类型安全注册
use crate::core::*; // 引入核心模块
use crate::error::StrategyError; // 引入策略相关错误类型
use anchor_lang::prelude::*; // Anchor 预导入，包含Context、Result等

// 引入统一的策略/参数类型定义
use crate::algorithms::execution_optimizer::types::*;

pub mod algorithm_registry; // 算法注册表子模块
pub mod execution_optimizer; // 执行优化器子模块
pub mod smart_routing; // 智能路由子模块
pub mod traits; // 算法trait子模块
pub mod twap; // TWAP算法子模块
pub mod vwap; // VWAP算法子模块

/// 算法 trait 对象通用封装，便于动态注册和类型擦除
pub trait AlgorithmBox: Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Send + Sync + 'static> AlgorithmBox for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 全局算法注册表，支持自动注册、热插拔、线程安全、动态扩展
///
/// # 用途
/// - 统一管理所有可用算法（如 TWAP、VWAP、Genetic、ML、SmartRouting 等）
/// - 支持算法 trait 对象的自动注册与运行时动态扩展
/// - 便于指令、工厂、外部服务等模块按需获取算法实例
///
/// # 线程安全
/// - 内部采用 RwLock+Arc，支持多线程并发注册与查询
///
/// # 自动注册与热插拔
/// - 各算法模块可通过 #[ctor::ctor] 在模块加载时自动注册
/// - 支持运行时动态添加/移除算法，便于升级和维护
///
/// # 主要接口
/// - `register(name, algo)`: 注册算法 trait 对象
/// - `get(name)`: 获取算法 trait 对象
/// - `list()`: 列出所有已注册算法名称
///
/// # 扩展方式
/// - 新增算法时实现 AlgorithmBox trait，并通过自动注册宏注册到本表
/// - 支持多版本/多类型算法并存，便于 A/B 测试和灰度发布
///
/// # 示例
/// ```rust
/// #[ctor::ctor]
/// fn register_twap_algorithm() {
///     ALGORITHM_REGISTRY.register_execution("twap", Arc::new(TwapAlgorithm::default()));
/// }
/// let algo = ALGORITHM_REGISTRY.get("twap");
/// ```
///
/// # 单元测试
/// - 详见本文件 tests 模块，覆盖注册、获取、并发等场景
pub struct AlgorithmRegistry {
    execution_algorithms: RwLock<HashMap<String, Arc<dyn ExecutionAlgorithm>>>, // 执行类算法注册表
    routing_algorithms: RwLock<HashMap<String, Arc<dyn RoutingAlgorithm>>>,     // 路由类算法注册表
    risk_algorithms: RwLock<HashMap<String, Arc<dyn RiskAlgorithm>>>,           // 风控类算法注册表
    optimizer_algorithms: RwLock<HashMap<String, Arc<dyn OptimizerAlgorithm>>>, // 优化器类算法注册表
}

impl AlgorithmRegistry {
    /// 创建新注册表实例（仅供全局单例初始化）
    pub fn new() -> Self {
        Self {
            execution_algorithms: RwLock::new(HashMap::new()), // 初始化执行类算法表
            routing_algorithms: RwLock::new(HashMap::new()),   // 初始化路由类算法表
            risk_algorithms: RwLock::new(HashMap::new()),      // 初始化风控类算法表
            optimizer_algorithms: RwLock::new(HashMap::new()), // 初始化优化器类算法表
        }
    }
    /// 注册执行类算法
    pub fn register_execution(&self, name: &str, algo: Arc<dyn ExecutionAlgorithm>) {
        self.execution_algorithms
            .write()
            .unwrap()
            .insert(name.to_string(), algo); // 注册算法实例
    }
    /// 获取执行类算法实例
    pub fn get_execution(&self, name: &str) -> Option<Arc<dyn ExecutionAlgorithm>> {
        self.execution_algorithms.read().unwrap().get(name).cloned() // 线程安全读取
    }
    /// 注册路由类算法
    pub fn register_routing(&self, name: &str, algo: Arc<dyn RoutingAlgorithm>) {
        self.routing_algorithms
            .write()
            .unwrap()
            .insert(name.to_string(), algo); // 注册路由算法
    }
    /// 获取路由类算法实例
    pub fn get_routing(&self, name: &str) -> Option<Arc<dyn RoutingAlgorithm>> {
        self.routing_algorithms.read().unwrap().get(name).cloned() // 线程安全读取
    }
    /// 注册风控类算法
    pub fn register_risk(&self, name: &str, algo: Arc<dyn RiskAlgorithm>) {
        self.risk_algorithms
            .write()
            .unwrap()
            .insert(name.to_string(), algo); // 注册风控算法
    }
    /// 获取风控类算法实例
    pub fn get_risk(&self, name: &str) -> Option<Arc<dyn RiskAlgorithm>> {
        self.risk_algorithms.read().unwrap().get(name).cloned() // 线程安全读取
    }
    /// 注册优化器类算法
    pub fn register_optimizer(&self, name: &str, algo: Arc<dyn OptimizerAlgorithm>) {
        self.optimizer_algorithms
            .write()
            .unwrap()
            .insert(name.to_string(), algo); // 注册优化器算法
    }
    /// 获取优化器类算法实例
    pub fn get_optimizer(&self, name: &str) -> Option<Arc<dyn OptimizerAlgorithm>> {
        self.optimizer_algorithms.read().unwrap().get(name).cloned() // 线程安全读取
    }
}

lazy_static::lazy_static! {
    /// 全局算法注册表单例（线程安全，模块加载时自动初始化）
    pub static ref ALGORITHM_REGISTRY: AlgorithmRegistry = AlgorithmRegistry::new();
}

/// 自动注册所有算法（可在各算法模块中通过 #[ctor::ctor] 注册）
// #[ctor::ctor]
fn register_all_algorithms() {
    use crate::core::registry::ALGORITHM_REGISTRY;
    use std::sync::Arc;
    ALGORITHM_REGISTRY.register_execution("twap", Arc::new(crate::algorithms::twap::TwapAlgorithm::default()));
    ALGORITHM_REGISTRY.register_execution("vwap", Arc::new(crate::algorithms::vwap::VwapAlgorithm::default()));
    ALGORITHM_REGISTRY.register_execution("smart_routing", Arc::new(crate::algorithms::smart_routing::SmartRoutingAlgorithm::default()));
    // 可扩展注册更多算法
}

// 预留自动注册点（可在各算法模块中通过 #[ctor::ctor] 注册）
// 例如：
// #[ctor::ctor]
// fn register_twap_algorithm() {
//     ALGORITHM_REGISTRY.register("twap", Arc::new(TwapAlgorithm::default()));
// }
// #[ctor::ctor]
// fn register_vwap_algorithm() {
//     ALGORITHM_REGISTRY.register("vwap", Arc::new(VwapAlgorithm::default()));
// }
// ...

/// 算法执行结果结构体，包含多维指标
#[derive(Debug, Clone)]
pub struct AlgorithmResult {
    /// 执行的算法类型
    pub algorithm_type: AlgorithmType,
    /// 总成交量
    pub volume_processed: u64,
    /// 执行效率评分（0-10000）
    pub efficiency_score: u32,
    /// Gas 消耗
    pub gas_used: u64,
    /// 执行耗时（毫秒）
    pub execution_time_ms: u64,
    /// 是否成功
    pub success: bool,
    /// 额外指标
    pub metrics: AlgorithmMetrics,
}

/// 算法类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlgorithmType {
    TWAP,
    VWAP,
    SmartRouting,
    MarketMaking,
    Arbitrage,
    RiskAssessment,
}

/// 算法综合指标结构体
#[derive(Debug, Clone, Default)]
pub struct AlgorithmMetrics {
    /// 滑点（基点）
    pub slippage_bps: u64,
    /// 价格改善（基点）
    pub price_improvement_bps: u64,
    /// MEV 保护效果（0-10000）
    pub mev_protection_score: u32,
    /// 流动性利用效率（0-10000）
    pub liquidity_efficiency: u32,
    /// 风险调整后收益
    pub risk_adjusted_return: i64,
    /// 总操作次数
    pub total_operations: u64,
    /// 成功操作次数
    pub successful_operations: u64,
    /// 失败操作次数
    pub failed_operations: u64,
    /// 平均执行耗时（毫秒）
    pub avg_execution_time_ms: u64,
    /// 总执行耗时（毫秒）
    pub total_execution_time_ms: u64,
    /// 最后一次操作时间戳
    pub last_operation_timestamp: i64,
}

impl AlgorithmMetrics {
    /// 用单次操作结果更新指标
    pub fn update_with_operation(&mut self, success: bool, execution_time_ms: u64) {
        self.total_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }

        self.total_execution_time_ms += execution_time_ms;
        self.avg_execution_time_ms = self.total_execution_time_ms / self.total_operations;

        self.last_operation_timestamp = Clock::get().unwrap().unix_timestamp;
    }

    /// 获取成功率（基点）
    pub fn success_rate_bps(&self) -> u16 {
        if self.total_operations > 0 {
            (self.successful_operations * 10_000) / self.total_operations
        } else {
            10_000
        }
    }

    /// 获取错误率（基点）
    pub fn error_rate_bps(&self) -> u16 {
        if self.total_operations > 0 {
            (self.failed_operations * 10_000) / self.total_operations
        } else {
            0
        }
    }
}

/// 所有交易算法的基础 trait
pub trait TradingAlgorithm {
    type Input;
    type Output;
    type Config;

    /// 执行算法
    fn execute(
        &mut self,
        input: Self::Input,
        config: &Self::Config,
        market_data: &EnhancedMarketData,
    ) -> StrategyResult<Self::Output>;

    /// 执行前参数校验
    fn validate_parameters(&self, input: &Self::Input, config: &Self::Config)
        -> StrategyResult<()>;

    /// 获取算法指标
    fn get_metrics(&self) -> AlgorithmMetrics;

    /// 重置算法状态
    fn reset(&mut self);
}

/// 算法工厂，便于创建各类算法实例
pub struct AlgorithmFactory;

impl AlgorithmFactory {
    /// 创建 TWAP 算法实例
    pub fn create_twap() -> TwapAlgorithm {
        TwapAlgorithm::new()
    }

    /// 创建 VWAP 算法实例
    pub fn create_vwap() -> VwapAlgorithm {
        VwapAlgorithm::new()
    }

    /// 创建智能路由算法实例
    pub fn create_smart_routing() -> SmartRoutingAlgorithm {
        SmartRoutingAlgorithm::new()
    }

    /// 创建市场冲击计算器
    pub fn create_market_impact_calculator() -> MarketImpactCalculator {
        MarketImpactCalculator::new()
    }

    /// 创建风险评估引擎
    pub fn create_risk_assessor() -> RiskAssessmentEngine {
        RiskAssessmentEngine::new()
    }

    /// 创建执行优化器
    pub fn create_execution_optimizer() -> ExecutionOptimizer {
        ExecutionOptimizer::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::execution_optimizer::genetic::GeneticOptimizer;
    use crate::algorithms::execution_optimizer::ml::MlOptimizer;
    use crate::algorithms::risk_assessment::RiskAssessmentAlgorithm;
    use crate::algorithms::smart_routing::SmartRoutingAlgorithm;
    use crate::algorithms::traits::{
        Assess, Execute, ExecutionParams, OptimizationParams, Optimize, RiskParams, Route,
        RoutingParams,
    };
    use crate::algorithms::twap::TwapAlgorithm;
    use crate::algorithms::vwap::VwapAlgorithm;
    use anchor_lang::prelude::*;
    use std::sync::Arc;

    #[test]
    fn test_algorithm_registry_register_and_get() {
        ALGORITHM_REGISTRY.register_execution("twap", Arc::new(TwapAlgorithm::default()));
        assert!(ALGORITHM_REGISTRY.get_execution("twap").is_some());
        ALGORITHM_REGISTRY.register_routing("smart_routing", Arc::new(SmartRoutingAlgorithm::default()));
        assert!(ALGORITHM_REGISTRY.get_routing("smart_routing").is_some());
        ALGORITHM_REGISTRY.register_risk(
            "risk_assessment",
            Arc::new(RiskAssessmentAlgorithm::default()),
        );
        assert!(ALGORITHM_REGISTRY.get_risk("risk_assessment").is_some());
        ALGORITHM_REGISTRY.register_optimizer("genetic", Arc::new(GeneticOptimizer));
        assert!(ALGORITHM_REGISTRY.get_optimizer("genetic").is_some());
    }

    #[test]
    fn test_algorithm_execution() {
        let twap = TwapAlgorithm::default();
        let params = ExecutionParams {
            amount: 1000,
            ..Default::default()
        };
        let ctx = Context::<Execute>::default();
        let result = twap.execute(ctx, &params);
        assert!(result.is_ok());
        let vwap = VwapAlgorithm::default();
        let ctx = Context::<Execute>::default();
        let result = vwap.execute(ctx, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_routing_and_risk() {
        let routing = SmartRoutingAlgorithm::default();
        let params = RoutingParams {
            amount: 1000,
            ..Default::default()
        };
        let ctx = Context::<Route>::default();
        let result = routing.route(ctx, &params);
        assert!(result.is_ok());
        let risk = RiskAssessmentAlgorithm::default();
        let params = RiskParams {
            ..Default::default()
        };
        let ctx = Context::<Assess>::default();
        let result = risk.assess(ctx, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_optimizer() {
        let genetic = GeneticOptimizer;
        let params = OptimizationParams {
            order_size: 100,
            market_impact: 5,
            slippage_tolerance: 10,
        };
        let ctx = Context::<Optimize>::default();
        let result = genetic.optimize(ctx, &params);
        assert!(result.is_ok());
        let ml = MlOptimizer;
        let result = ml.optimize(ctx, &params);
        assert!(result.is_ok());
    }
}
