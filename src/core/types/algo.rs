//!
//! algo.rs - 算法参数类型定义
//!
//! 本文件定义了AlgoParams结构体，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 算法参数结构体
/// - 适用于所有算法融合指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AlgoParams {
    /// 算法名称
    pub algo_name: String,
    /// 算法参数序列化数据
    pub params: Vec<u8>,
} 