// ========================= Oracle 适配器工厂实现 =========================
// 本模块实现 Oracle 适配器的类型枚举、注册、获取、批量初始化等工厂方法，
// 每个 struct、enum、trait、方法、参数、用途、边界、Anchor 相关点均有详细注释。
// - 设计意图：极致可插拔、最小功能单元、统一接口、线程安全、Anchor集成友好
/*!
 * Oracle Adapter Factory
 *
 * 工厂方法，按类型动态获取Oracle适配器实例。
 * - 设计意图：便于多预言机动态扩展、统一注册、批量初始化、Anchor集成友好
 */

use super::chainlink_adapter::ChainlinkOracle; // Chainlink适配器实现
use super::traits::OracleAdapter;              // OracleAdapter trait
use anchor_lang::prelude::*;                   // Anchor预导入
use std::collections::HashMap;                 // HashMap用于类型到实例映射
use std::sync::Mutex;                          // Mutex保证线程安全

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// OracleType 枚举
/// - 枚举所有支持的主流Oracle类型
/// - 用于工厂方法动态获取对应Oracle适配器实例
pub enum OracleType {
    /// Pyth 预言机
    Pyth,
    /// Switchboard 预言机
    Switchboard,
    /// Chainlink 预言机
    Chainlink,
}

lazy_static::lazy_static! {
    /// 全局Oracle适配器注册表（类型到实例的映射，线程安全）
    /// - 使用Mutex保证多线程安全
    /// - 设计意图：保证所有线程/模块可安全注册和获取Oracle适配器
    static ref ORACLE_ADAPTER_REGISTRY: Mutex<HashMap<OracleType, Box<dyn OracleAdapter + Send + Sync>>> = Mutex::new(HashMap::new());
}

/// 注册Oracle适配器到全局注册表
/// - oracle_type: Oracle类型
/// - adapter: 适配器实例
/// - 设计意图：支持运行时热插拔、动态扩展
pub fn register_oracle_adapter(
    oracle_type: OracleType,
    adapter: Box<dyn OracleAdapter + Send + Sync>,
) {
    ORACLE_ADAPTER_REGISTRY
        .lock()
        .unwrap()
        .insert(oracle_type, adapter);
}

/// 获取指定类型的Oracle适配器实例
/// - oracle_type: Oracle类型
/// - 返回：适配器实例（可选）
/// - 设计意图：便于服务端/链上/前端动态获取适配器
pub fn get_oracle_adapter(oracle_type: OracleType) -> Option<Box<dyn OracleAdapter + Send + Sync>> {
    ORACLE_ADAPTER_REGISTRY
        .lock()
        .unwrap()
        .get(&oracle_type)
        .cloned()
}

/// 工厂初始化，注册所有主流Oracle adapter
/// - 建议在程序启动时调用，确保所有Oracle适配器可用
/// - 支持后续扩展新Oracle类型
/// - 设计意图：极简批量注册，便于持续集成和测试
pub fn init_oracle_adapters() {
    register_oracle_adapter(OracleType::Pyth, Box::new(PythOracle));
    register_oracle_adapter(OracleType::Switchboard, Box::new(SwitchboardOracle));
    register_oracle_adapter(OracleType::Chainlink, Box::new(ChainlinkOracle));
}
