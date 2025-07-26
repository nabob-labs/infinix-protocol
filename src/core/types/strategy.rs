//!
//! strategy.rs - 策略参数类型定义
//!
//! 本文件定义了StrategyParams结构体，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 策略参数结构体
/// - 适用于所有策略融合指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StrategyParams {
    /// 策略名称
    pub strategy_name: String,
    /// 策略参数序列化数据
    pub params: Vec<u8>,
} 