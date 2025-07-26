//!
//! dex.rs - DEX参数类型定义
//!
//! 本文件定义了DexParams结构体，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// DEX 参数结构体
/// - 适用于所有 DEX/AMM 指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DexParams {
    /// DEX 名称
    pub dex_name: String,
    /// DEX 参数序列化数据
    pub params: Vec<u8>,
} 