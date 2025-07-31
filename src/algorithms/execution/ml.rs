//! 机器学习（ML）算法执行器实现模块
//! 实现 ExecutionStrategy trait，支持基于线性回归的订单执行优化。
//! 支持 Anchor 自动注册，便于工厂/注册表动态调用。

use crate::algorithms::traits::{ExecutionStrategy, ExecutionResult, AlgorithmError}; // 引入执行策略 trait 及相关类型，保证算法接口统一
use crate::core::types::AlgoParams; // 引入通用算法参数类型，便于算法通用化和参数安全
use crate::core::adapter::AdapterTrait; // 引入适配器 trait，便于统一管理和注册
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保与 Anchor 生态兼容

/// ML 算法执行器实现结构体
pub struct MlImpl; // 主结构体，无状态实现，提升安全性和可复用性

/// ExecutionStrategy trait 实现
impl ExecutionStrategy for MlImpl {
    /// 执行 ML 算法主入口
    /// - 参数 ctx: Anchor 上下文
    /// - 参数 params: 算法参数（需序列化为 (order_size, market_impact, slippage_tolerance)）
    /// - 返回 ExecutionResult，包含优化后成交量、预期成本等
    fn execute(&self, _ctx: Context<crate::algorithms::traits::Execute>, params: &AlgoParams) -> anchor_lang::Result<ExecutionResult> {
        // 解析 AlgoParams，获取 order_size、market_impact、slippage_tolerance
        let (order_size, market_impact, slippage_tolerance): (u64, u64, u64) = bincode::deserialize(&params.params)
            .map_err(|_| AlgorithmError::InvalidInput)?; // 反序列化参数，错误则返回 InvalidInput，防止恶意输入
        // 校验参数有效性，订单量和滑点容忍度必须大于0
        if order_size == 0 || slippage_tolerance == 0 {
            return Err(AlgorithmError::InvalidInput.into()); // 输入参数校验，防止无效或恶意调用
        }
        // 生产级 ML 算法核心流程（以线性回归为例）
        // features: 特征向量，分别为订单量、市场冲击、滑点容忍度
        let features = vec![order_size as f64, market_impact as f64, slippage_tolerance as f64]; // 构造特征向量
        // weights: 线性回归权重，业务可根据实际模型调整
        let weights = vec![0.7, 0.2, 0.1]; // 线性回归权重，需根据实际业务/模型训练结果调整
        // bias: 偏置项
        let bias = 10.0; // 偏置项，提升模型灵活性
        // prediction: 预测优化后成交量
        let mut prediction = bias; // 初始化预测值为偏置项
        for (f, w) in features.iter().zip(weights.iter()) {
            prediction += f * w; // 线性回归加权求和
        }
        // 优化后成交量需在[1, order_size]区间
        let optimized_size = prediction.max(1.0).min(order_size as f64) as u64; // 保证成交量合法，防止溢出/下溢
        // 预期成本，实际应根据市场行情/模型输出
        let expected_cost = optimized_size * 1_000_000; // 简化示例，实际应为动态计算
        Ok(ExecutionResult {
            optimized_size, // 优化后成交量，类型安全
            expected_cost,  // 总成交成本，便于链上链下审计
        })
    }
    /// 算法名称
    fn name(&self) -> &'static str { "ML" } // 返回算法名称，便于注册表/工厂统一管理
}

/// AdapterTrait 实现，便于统一管理和注册
impl AdapterTrait for MlImpl {
    /// 获取算法名称
    fn name(&self) -> &'static str { "ml" } // 返回适配器名称
    /// 获取算法版本
    fn version(&self) -> &'static str { "1.0.0" } // 返回适配器版本，便于升级和兼容性管理
    /// 支持的资产类型
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] } // 支持的资产类型，便于资产适配
    /// 算法状态
    fn status(&self) -> Option<String> { Some("active".to_string()) } // 算法状态，便于运维监控
}

/// Anchor 自动注册宏，模块加载时自动注册到工厂
#[ctor::ctor]
fn auto_register_ml_impl() {
    // 创建 ML 算法适配器实例
    let adapter = MlImpl; // 实例化无状态适配器
    // 获取全局 ADAPTER_FACTORY 工厂锁
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap(); // 获取全局工厂互斥锁，保证线程安全
    // 注册适配器到工厂，便于运行时动态获取
    factory.register(adapter); // 注册适配器，支持热插拔和动态扩展
} 