//!
//! Offchain Integration Oracles Module
//!
//! 本模块实现离线集成服务的预言机相关逻辑，支持价格查询、TWAP/VWAP 等功能，确保与链下预言机交互的合规性与安全性。

/// 预言机查询参数结构体。
#[derive(Debug, Clone)]
pub struct OracleQueryParams {
    pub token: String,         // 查询代币
}

/// 预言机查询结果结构体。
#[derive(Debug, Clone)]
pub struct OracleQueryResult {
    pub price: u64,            // 查询价格
    pub timestamp: u64,        // 价格时间戳
}

/// 预言机适配器 trait。
pub trait OracleAdapter {
    /// 查询价格。
    fn get_price(&self, params: &OracleQueryParams) -> OracleQueryResult;
    /// 查询 TWAP。
    fn get_twap(&self, params: &OracleQueryParams) -> OracleQueryResult;
    /// 查询 VWAP。
    fn get_vwap(&self, params: &OracleQueryParams) -> OracleQueryResult;
}
