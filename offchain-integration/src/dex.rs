//!
//! Offchain Integration DEX Module
//!
//! 本模块实现离线集成服务的 DEX 相关逻辑，支持报价、交易等功能，确保与链下 DEX 交互的合规性与安全性。

/// DEX 交易参数结构体。
#[derive(Debug, Clone)]
pub struct DexTradeParams {
    pub from_token: String,    // 源代币
    pub to_token: String,      // 目标代币
    pub amount: u64,           // 交易数量
}

/// DEX 交易结果结构体。
#[derive(Debug, Clone)]
pub struct DexTradeResult {
    pub success: bool,         // 交易是否成功
    pub amount_out: u64,       // 实际获得数量
    pub tx_hash: String,       // 交易哈希
}

/// DEX 适配器 trait。
pub trait DexAdapter {
    /// 查询报价。
    fn quote(&self, params: &DexTradeParams) -> u64;
    /// 执行交易。
    fn swap(&self, params: &DexTradeParams) -> DexTradeResult;
}
