//!
//! adapter.rs - 高级策略适配器实现
//!
//! 本文件实现AdvancedStrategyExecutor的AdapterTrait实现与自动注册函数，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::core::adapter::AdapterTrait;
use crate::strategies::advanced_strategies::executor::AdvancedStrategyExecutor;
use anchor_lang::prelude::*;

impl AdapterTrait for AdvancedStrategyExecutor {
    /// 返回适配器名称。
    fn name(&self) -> &'static str { "advanced_strategy_executor" }
    /// 返回适配器版本。
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表。
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器当前状态。
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

/// 自动注册高级策略执行器到全局工厂。
// #[ctor::ctor]
pub fn auto_register_advanced_strategy_executor() {
    // 实例化高级策略执行器。
    let adapter = AdvancedStrategyExecutor;
    // 获取全局适配器工厂的可变引用。
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    // 注册适配器。
    factory.register(adapter);
} 