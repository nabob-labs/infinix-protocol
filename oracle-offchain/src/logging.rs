//!
//! Oracle Offchain Logging Module
//!
//! 本模块实现预言机离线服务日志工具，支持标准化日志输出，便于调试与运维。

/// 日志工具结构体。
pub struct OracleLogger;

impl OracleLogger {
    /// 输出信息日志。
    pub fn info(msg: &str) {
        println!("[INFO] {}", msg);
    }
    /// 输出错误日志。
    pub fn error(msg: &str) {
        eprintln!("[ERROR] {}", msg);
    }
} 