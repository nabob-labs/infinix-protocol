//!
//! DEX Logging Module
//!
//! 本模块实现 DEX 相关日志工具，支持标准化日志输出，便于调试、监控与合规审计。

/// DEX 日志工具结构体。
pub struct DexLogger;

impl DexLogger {
    /// 输出信息日志。
    pub fn info(msg: &str) {
        // 标准输出信息日志，便于开发与运维追踪
        println!("[DEX][INFO] {}", msg);
    }
    /// 输出警告日志。
    pub fn warn(msg: &str) {
        // 标准输出警告日志，便于监控潜在风险
        println!("[DEX][WARN] {}", msg);
    }
    /// 输出错误日志。
    pub fn error(msg: &str) {
        // 标准输出错误日志，便于故障排查与合规审计
        eprintln!("[DEX][ERROR] {}", msg);
    }
} 