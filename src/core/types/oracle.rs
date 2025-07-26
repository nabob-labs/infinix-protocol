//!
//! oracle.rs - 预言机参数类型定义
//!
//! 本文件定义了OracleParams结构体，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;

/// 预言机参数结构体
/// - 适用于所有 Oracle 指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OracleParams {
    /// 预言机名称
    pub oracle_name: String,
    /// 预言机参数序列化数据
    pub params: Vec<u8>,
} 