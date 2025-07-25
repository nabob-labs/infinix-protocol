//!
//! Offchain Integration Traits Module
//!
//! 本模块定义离线集成服务的通用 trait，规范 DEX、预言机等接口，确保可扩展性、合规性与可维护性。

/// 离线 DEX 适配器 trait。
pub trait OffchainDexAdapter {
    /// 获取适配器名称。
    fn name(&self) -> &str;
    /// 查询报价。
    fn quote(&self, from_token: &str, to_token: &str, amount: u64) -> u64;
    /// 执行交易。
    fn swap(&self, from_token: &str, to_token: &str, amount: u64) -> bool;
}

/// 离线预言机适配器 trait。
pub trait OffchainOracleAdapter {
    /// 获取适配器名称。
    fn name(&self) -> &str;
    /// 查询价格。
    fn get_price(&self, token: &str) -> u64;
}
