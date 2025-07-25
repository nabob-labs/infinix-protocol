use anchor_lang::prelude::*; // Anchor预导入，包含msg!等

// ========================= Oracles 日志模块实现 =========================
// 本模块为 Oracles 适配器提供标准化链上事件、错误、指标日志输出，
// 每个函数、参数、用途、边界、Anchor 相关点均有详细注释。
// - 设计意图：极简、统一、可插拔，便于所有Oracle适配器/工厂/注册表复用

/// 记录预言机模块事件日志
/// - 用于链上输出关键业务事件，便于调试、监控、审计
/// - 设计意图：极简、统一、可插拔，便于所有Oracle适配器/工厂/注册表复用
///
/// # 参数
/// * `context` - 业务上下文描述（如预言机名称、操作类型等）
/// * `event` - 事件内容（如get_price、get_twap等）
pub fn log_event(context: &str, event: &str) {
    msg!("[ORACLE][EVENT] {}: {}", context, event); // Anchor链上日志输出
}

/// 记录预言机模块错误日志
/// - 用于链上输出错误信息，便于排查、监控、审计
/// - 设计意图：极简、统一、可插拔，便于所有Oracle适配器/工厂/注册表复用
///
/// # 参数
/// * `context` - 业务上下文描述
/// * `error` - 错误内容
pub fn log_error(context: &str, error: &str) {
    msg!("[ORACLE][ERROR] {}: {}", context, error); // Anchor链上日志输出
}

/// 记录预言机模块指标日志
/// - 用于链上输出关键指标（如价格、更新时间等），便于性能分析、监控、审计
/// - 设计意图：极简、统一、可插拔，便于所有Oracle适配器/工厂/注册表复用
///
/// # 参数
/// * `context` - 业务上下文描述
/// * `metric` - 指标名称
/// * `value` - 指标数值
pub fn log_metric(context: &str, metric: &str, value: u64) {
    msg!("[ORACLE][METRIC] {}: {}={}", context, metric, value); // Anchor链上日志输出
}
