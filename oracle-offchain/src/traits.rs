//!
//! Oracle Offchain Traits Module
//!
//! 本模块定义预言机离线服务的通用 trait，规范适配器、工厂、API 客户端等接口，确保可扩展性、合规性与可维护性。

/// 预言机离线适配器 trait。
pub trait OracleOffchainAdapter {
    /// 获取适配器名称。
    fn name(&self) -> &str;
    /// 查询价格。
    fn get_price(&self, token: &str) -> u64;
}

/// 预言机工厂 trait。
pub trait OracleFactoryTrait {
    /// 创建适配器实例。
    fn create_adapter(&self) -> Box<dyn OracleOffchainAdapter>;
}

/// REST 客户端 trait。
pub trait RestClientTrait {
    /// 发送 GET 请求。
    fn get(&self, endpoint: &str) -> String;
    /// 发送 POST 请求。
    fn post(&self, endpoint: &str, body: &str) -> String;
}
