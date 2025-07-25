//!
//! DEX Offchain Traits Module
//!
//! 本模块定义 DEX 离线服务的通用 trait，规范适配器、工厂、API 客户端等接口，确保可扩展性、合规性与可维护性。

/// DEX 离线适配器 trait。
pub trait DexOffchainAdapter {
    /// 获取适配器名称。
    fn name(&self) -> &str;
    /// 获取适配器版本。
    fn version(&self) -> &str;
    /// 查询报价。
    fn quote(&self, from_token: &str, to_token: &str, amount: u64) -> u64;
    /// 执行交易。
    fn swap(&self, from_token: &str, to_token: &str, amount: u64) -> bool;
}

/// DEX 工厂 trait。
pub trait DexFactoryTrait {
    /// 创建适配器实例。
    fn create_adapter(&self) -> Box<dyn DexOffchainAdapter>;
}

/// REST 客户端 trait。
pub trait RestClientTrait {
    /// 发送 GET 请求。
    fn get(&self, endpoint: &str) -> String;
    /// 发送 POST 请求。
    fn post(&self, endpoint: &str, body: &str) -> String;
}
