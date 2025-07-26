// ========================= Oracle 适配器核心实现 =========================
// 本模块标准化所有 Oracle 适配器 trait、注册表、宏、事件、账户声明等，
// 每个 trait、struct、enum、宏、参数、用途、边界、Anchor 相关点均有详细注释。
// - 设计意图：极致可插拔、最小功能单元、统一接口、Anchor集成友好、可观测性、可维护性、可审计性

/*!
 * Oracle适配器Trait（标准化、可扩展）
 *
 * 统一抽象所有预言机链上适配器接口，支持多资产、多市场、多功能类型参数化，便于生产级集成与扩展。
 *
 * # 设计原则
 * - 每个trait只定义最小功能单元接口
 * - 参数泛型化，支持多资产/多市场/多功能扩展
 * - 详细注释和用法示例
 * - 统一注册机制，adapter自动注册宏，注册表操作有事件日志
 */

use anchor_lang::prelude::*; // Anchor预导入，包含Result、Accounts、msg!等
use crate::core::adapter::AdapterTrait; // 适配器元信息trait，统一接口
use crate::core::types::OracleParams;   // 预言机参数类型

/// 预言机适配器Trait（标准化、可扩展）
/// - 统一所有链上预言机的适配接口
/// - 支持现价、TWAP、VWAP等多种查询
/// - 设计意图：所有预言机适配器必须实现，便于统一聚合、扩展、测试
pub trait OracleAdapter: AdapterTrait {
    /// 获取现价
    /// - params: OracleParams结构体，包含资产、oracle名称等
    /// - 返回：OraclePriceResult结构体
    fn get_price(&self, params: &OracleParams) -> anchor_lang::Result<OraclePriceResult>;
    /// 获取TWAP
    /// - params: OracleParams结构体
    /// - 返回：OracleTwapResult结构体
    fn get_twap(&self, params: &OracleParams) -> anchor_lang::Result<OracleTwapResult>;
    /// 获取VWAP
    /// - params: OracleParams结构体
    /// - 返回：OracleVwapResult结构体
    fn get_vwap(&self, params: &OracleParams) -> anchor_lang::Result<OracleVwapResult>;
    /// 支持的资产类型
    /// - 返回：资产名称列表
    fn supported_assets(&self) -> Vec<String> { vec![] }
    /// 支持的市场类型
    /// - 返回：市场类型名称列表
    fn supported_markets(&self) -> Vec<String> { vec![] }
    /// 预言机adapter类型
    /// - 返回：OracleAdapterType枚举
    fn adapter_type(&self) -> OracleAdapterType { OracleAdapterType::Other }
}

/// 预言机adapter类型枚举
/// - 标识适配器对接的预言机类型
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum OracleAdapterType {
    /// Pyth预言机
    Pyth,         // Pyth预言机
    /// Switchboard预言机
    Switchboard,  // Switchboard预言机
    /// Chainlink预言机
    Chainlink,    // Chainlink预言机
    /// 其他类型
    Other,        // 其他
}

/// 现价结果结构体
/// - 记录一次get_price操作的结果
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct OraclePriceResult {
    /// 现价
    pub price: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
    /// 预言机名称
    pub oracle_name: String,
}

/// TWAP结果结构体
/// - 记录一次get_twap操作的结果
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct OracleTwapResult {
    /// TWAP数值
    pub twap: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
    /// 预言机名称
    pub oracle_name: String,
}

/// VWAP结果结构体
/// - 记录一次get_vwap操作的结果
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct OracleVwapResult {
    /// VWAP数值
    pub vwap: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
    /// 预言机名称
    pub oracle_name: String,
}

// === Anchor账户声明（可扩展） ===
#[derive(Accounts)]
/// get_price指令所需账户声明（可根据具体预言机扩展）
/// - 设计意图：为 Anchor 指令集成提供账户校验模板，便于各预言机自定义扩展
pub struct GetPrice<'info> {
    // 相关账户声明（如oracle账户、用户等）
    // - 具体预言机可通过扩展本结构体，声明所需账户
}

#[derive(Accounts)]
/// get_twap指令所需账户声明
/// - 设计意图：TWAP查询场景下的账户声明模板，便于 Anchor 校验
pub struct GetTwap<'info> {
    // 相关账户声明
    // - 具体预言机可扩展
}

#[derive(Accounts)]
/// get_vwap指令所需账户声明
/// - 设计意图：VWAP查询场景下的账户声明模板，便于 Anchor 校验
pub struct GetVwap<'info> {
    // 相关账户声明
    // - 具体预言机可扩展
}

// === 注册表与自动注册宏 ===
use std::collections::HashMap; // HashMap：适配器名称到实例的映射
use std::sync::{Arc, RwLock}; // Arc/RwLock：线程安全全局注册表

/// 预言机适配器注册表
/// - 支持动态注册、注销、查询、事件日志
/// - 线程安全，适用于多线程环境
pub struct OracleAdapterRegistry {
    /// 适配器名称到实例的映射（线程安全）
    adapters: RwLock<HashMap<String, Arc<dyn OracleAdapter + Send + Sync>>>,
}

impl OracleAdapterRegistry {
    /// 创建新注册表
    /// 返回：OracleAdapterRegistry对象
    /// - 设计意图：便于多实例/测试环境下灵活创建注册表
    pub fn new() -> Self {
        Self { adapters: RwLock::new(HashMap::new()) }
    }
    /// 注册适配器
    /// 参数：name 适配器名称，adapter 适配器实例
    /// - 设计意图：支持插件式热插拔，便于运行时动态扩展
    pub fn register(&self, name: &str, adapter: Arc<dyn OracleAdapter + Send + Sync>) {
        self.adapters.write().unwrap().insert(name.to_string(), adapter);
        // 事件日志，便于链上/本地可观测性
        msg!("[OracleAdapterRegistry] Registered adapter: {}", name);
    }
    /// 注销适配器
    /// 参数：name 适配器名称
    /// - 设计意图：支持运行时卸载，便于测试/权限管理
    pub fn unregister(&self, name: &str) {
        self.adapters.write().unwrap().remove(name);
        msg!("[OracleAdapterRegistry] Unregistered adapter: {}", name);
    }
    /// 查询适配器
    /// 参数：name 适配器名称
    /// 返回：Option<Arc<dyn OracleAdapter + Send + Sync>>
    /// - 设计意图：便于服务端/前端/链上动态获取适配器实例
    pub fn get(&self, name: &str) -> Option<Arc<dyn OracleAdapter + Send + Sync>> {
        self.adapters.read().unwrap().get(name).cloned()
    }
    /// 列出所有已注册适配器名称
    /// 返回：适配器名称字符串列表
    /// - 设计意图：便于前端/服务端动态发现所有可用Oracle
    pub fn list(&self) -> Vec<String> {
        self.adapters.read().unwrap().keys().cloned().collect()
    }
}

/// 全局预言机适配器注册表（单例）
/// - 使用lazy_static实现全局唯一实例
lazy_static::lazy_static! {
    pub static ref ORACLE_ADAPTER_REGISTRY: OracleAdapterRegistry = OracleAdapterRegistry::new();
}

/// 自动注册预言机适配器宏
/// - 用于在模块加载时自动注册adapter，便于插件式扩展
/// - 参数：$name 适配器名称，$adapter 适配器实例
/// - 设计意图：极简插件式扩展，支持 #[ctor::ctor] 自动注册
#[macro_export]
macro_rules! auto_register_oracle_adapter {
    ($name:expr, $adapter:expr) => {
        // #[ctor::ctor]
        fn auto_register() {
            $crate::oracles::adapter::ORACLE_ADAPTER_REGISTRY.register($name, std::sync::Arc::new($adapter));
        }
    };
}

// === 示例实现 ===
/// MockOracleAdapter为OracleAdapter trait的测试实现，便于单元测试
pub struct MockOracleAdapter;
impl AdapterTrait for MockOracleAdapter {
    /// 返回适配器名称
    fn name(&self) -> &'static str { "mock_oracle" }
    /// 返回适配器版本
    fn version(&self) -> &'static str { "1.0.0" }
    /// 返回支持的资产列表
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回适配器状态
    fn status(&self) -> Option<String> { Some("active".to_string()) }
}
impl OracleAdapter for MockOracleAdapter {
    /// 模拟获取现价，返回固定价格
    fn get_price(&self, _params: &OracleParams) -> anchor_lang::Result<OraclePriceResult> {
        Ok(OraclePriceResult {
            price: 1_000_000,
            last_updated: 1_700_000_000,
            oracle_name: "mock_oracle".to_string(),
        })
    }
    /// 模拟获取TWAP，返回固定数值
    fn get_twap(&self, _params: &OracleParams) -> anchor_lang::Result<OracleTwapResult> {
        Ok(OracleTwapResult {
            twap: 1_000_000,
            last_updated: 1_700_000_000,
            oracle_name: "mock_oracle".to_string(),
        })
    }
    /// 模拟获取VWAP，返回固定数值
    fn get_vwap(&self, _params: &OracleParams) -> anchor_lang::Result<OracleVwapResult> {
        Ok(OracleVwapResult {
            vwap: 1_000_000,
            last_updated: 1_700_000_000,
            oracle_name: "mock_oracle".to_string(),
        })
    }
    /// 返回支持的资产列表
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] }
    /// 返回支持的市场类型
    fn supported_markets(&self) -> Vec<String> { vec!["spot".to_string()] }
    /// 返回适配器类型
    fn adapter_type(&self) -> OracleAdapterType { OracleAdapterType::Other }
}

// 自动注册MockOracleAdapter
// 便于开发环境自动集成
auto_register_oracle_adapter!("mock_oracle", MockOracleAdapter);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mock_oracle_adapter() {
        let adapter = MockOracleAdapter;
        let params = OracleParams {
            asset: Default::default(),
            oracle_name: "mock_oracle".to_string(),
            price: 0,
        };
        let result = adapter.get_price(&params).unwrap();
        assert_eq!(result.price, 1_000_000);
        assert_eq!(result.oracle_name, "mock_oracle");
    }
}
