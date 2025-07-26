//!
//! adapter.rs - 权重策略适配器实现
//!
//! 本文件实现WeightStrategyAdapter及其AdapterTrait实现、自动注册函数，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::core::adapter::AdapterTrait;
use anchor_lang::prelude::*;

/// 权重策略适配器结构体
pub struct WeightStrategyAdapter;

impl AdapterTrait for WeightStrategyAdapter {
    /// 返回适配器名称
    fn name(&self) -> &'static str { "weight_strategy" }
    /// 返回适配器版本
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产类型
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器状态
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}

/// 自动注册权重策略适配器
pub fn auto_register_weight_strategy_adapter() {
    // 生产环境应注册到全局适配器注册表
    // 这里只做示例
} 