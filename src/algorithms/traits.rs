//!
//! 算法 Trait（标准化、可扩展）
//!
//! 统一抽象所有算法/策略接口，支持多资产、多市场、多功能类型参数化，便于热插拔和动态扩展。
//!
//! # 设计原则
//! - 每个 trait 只定义最小功能单元接口
//! - 参数泛型化，支持多资产/多市场/多功能扩展
//! - 详细注释和用法示例
//! - 便于 adapter/algorithm/strategy 热插拔和动态注册

use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保trait与Anchor兼容
use crate::core::adapter::AdapterTrait;
use crate::core::types::{AlgoParams, TradeParams, StrategyParams};
// use crate::core::types::{TradeParams, BatchTradeParams, AlgoParams, StrategyParams}; // 引入通用参数类型，便于算法通用化 - 暂时注释掉

/// 通用算法 trait，所有算法/策略都应实现
/// - 继承 AdapterTrait，便于统一管理
pub trait Algorithm: AdapterTrait {
    /// 执行算法的最小功能单元接口
    /// - 参数 params: 算法参数
    /// - 返回 ExecutionResult，包含执行结果
    fn execute(&self, params: &AlgoParams) -> anchor_lang::Result<ExecutionResult>; // 算法主入口，参数类型安全
    /// 算法名称
    fn name(&self) -> &'static str;
    /// 算法支持的资产类型（如 SOL、USDC、ETF、RWA 等）
    fn supported_assets(&self) -> Vec<String> { vec![] } // 默认空，子类可重载
    /// 算法支持的市场类型（如现货、期货、DEX、AMM 等）
    fn supported_markets(&self) -> Vec<String> { vec![] } // 默认空，子类可重载
    /// 算法功能类型（如执行、路由、风控、优化等）
    fn algorithm_type(&self) -> AlgorithmType { AlgorithmType::Other } // 默认 Other，子类可重载
}

/// 执行类算法 trait（如 TWAP、VWAP、Genetic、ML 等）
pub trait ExecutionStrategy: Algorithm {
    /// 执行算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 算法参数
    /// - 返回 ExecutionResult
    fn execute(&self, ctx: Context<Execute>, params: &AlgoParams) -> anchor_lang::Result<ExecutionResult>; // Anchor 上下文+参数，类型安全
    /// 算法名称
    fn name(&self) -> &'static str;
}

/// 路由类算法 trait（如智能路由、跨市场套利等）
pub trait RoutingStrategy: Algorithm {
    /// 路由算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 交易参数
    /// - 返回 RoutingResult
    fn route(&self, ctx: Context<Route>, params: &TradeParams) -> anchor_lang::Result<RoutingResult>; // Anchor 上下文+参数，类型安全
}

/// 风控类算法 trait（如风险评估、风控管理等）
pub trait RiskManagement: Algorithm {
    /// 风控算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 策略参数
    /// - 返回 RiskResult
    fn assess(&self, ctx: Context<AssessRisk>, params: &StrategyParams) -> anchor_lang::Result<RiskResult>; // Anchor 上下文+参数，类型安全
}

/// 优化器类算法 trait（如执行优化、成本最小化等）
pub trait OptimizerStrategy: Algorithm {
    /// 优化算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 算法参数
    /// - 返回 OptimizerResult
    fn optimize(&self, ctx: Context<Optimize>, params: &AlgoParams) -> anchor_lang::Result<OptimizerResult>; // Anchor 上下文+参数，类型安全
}

/// 算法类型枚举，便于注册表/工厂统一管理
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)] // 派生常用trait，便于序列化、调试、判等
pub enum AlgorithmType {
    Execution,   // 执行类
    Routing,     // 路由类
    Risk,        // 风控类
    Optimizer,   // 优化类
    Other,       // 其他
}

// === 统一算法参数和结果结构体 ===

/// 执行算法结果结构体
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)] // 派生常用trait，便于序列化、调试、复制
pub struct ExecutionResult {
    pub optimized_size: u64,   // 优化后成交量
    pub expected_cost: u64,    // 预期成本
}

/// 路由算法结果结构体
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)] // 派生常用trait，便于序列化、调试、复制
pub struct RoutingResult {
    pub best_dex: String,      // 最优 DEX 名称
    pub expected_out: u64,     // 预期输出
}

/// 风控算法结果结构体
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)] // 派生常用trait，便于序列化、调试、复制
pub struct RiskResult {
    pub risk_score: u8,        // 风险评分
    pub is_acceptable: bool,   // 是否可接受
}

/// 优化器算法结果结构体
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)] // 派生常用trait，便于序列化、调试、复制
pub struct OptimizerResult {
    pub optimized_value: u64,  // 优化后数值
    pub achieved_cost: u64,    // 实际成本
}

/// 算法错误类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)] // 派生常用trait，便于序列化、调试、判等
pub enum AlgorithmError {
    InvalidInput,              // 输入无效
    CalculationError,          // 计算错误
    Other(String),             // 其他错误
}

// === Anchor账户声明（可扩展） ===

/// 执行算法指令账户参数结构体
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct Execute<'info> {
    // 相关账户声明（可根据实际业务扩展）
}

/// 风控算法指令账户参数结构体
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct AssessRisk<'info> {
    // 相关账户声明
}

/// 路由算法指令账户参数结构体
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct Route<'info> {
    // 相关账户声明
}

/// 优化算法指令账户参数结构体
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct Optimize<'info> {
    // 相关账户声明
}

// === 示例实现 ===

/// Mock 执行算法实现（用于测试和演示）
pub struct MockExecutionAlgorithm; // 演示用mock结构体

impl ExecutionStrategy for MockExecutionAlgorithm {
    fn execute(
        &self,
        _ctx: Context<crate::algorithms::traits::Execute>,
        params: &AlgoParams,
    ) -> anchor_lang::Result<ExecutionResult> {
        Ok(ExecutionResult {
            optimized_size: params.order_size, // 直接返回输入量
            expected_cost: params.order_size * 1_000_000, // 假定每单位成本1000000
        })
    }
    fn name(&self) -> &'static str {
        "MockExecution" // 算法名称
    }
}

#[cfg(test)] // 测试模块，仅编译测试时启用
mod tests {
    use super::*; // 引入父模块所有定义
    use anchor_lang::prelude::*; // Anchor预导入，便于测试账户等
    struct DummyAccounts; // mock账户结构体
    impl anchor_lang::prelude::Key for DummyAccounts {
        fn key(&self) -> Pubkey {
            Pubkey::default() // 返回默认公钥
        }
    }
    #[test] // 单元测试注解
    fn test_mock_execution_algorithm() {
        let algo = MockExecutionAlgorithm; // 实例化mock算法
        let params = AlgoParams {
            order_size: 100, // 测试订单量
            market_impact: 0, // 市场冲击
            slippage_tolerance: 100, // 滑点容忍
        };
        let ctx = Context::new(DummyAccounts, vec![]); // 构造mock Anchor上下文
        let result = algo.execute(ctx, &params).unwrap(); // 执行mock算法
        assert_eq!(result.optimized_size, 100); // 校验mock算法输出
        assert_eq!(result.expected_cost, 100_000_000); // 校验mock算法输出
    }
}
