//!
//! traits.rs - 策略Trait与执行接口定义
//!
//! 本文件定义了策略Trait与执行接口，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
// use crate::core::adapter; // 暂时注释掉

/// 策略参数结构体（示例，可根据实际业务扩展）
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct StrategyParams {
    // 具体参数字段根据业务需求定义
}

/// 策略执行结果结构体（示例，可根据实际业务扩展）
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct StrategyResult {
    // 具体结果字段根据业务需求定义
}

/// 策略Trait，所有策略需实现该接口
pub trait Strategy: AdapterTrait {
    /// 执行策略
    fn execute(&self, params: &StrategyParams) -> anchor_lang::Result<StrategyResult>;
} 