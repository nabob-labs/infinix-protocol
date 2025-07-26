/// 资产相关事件模块
pub mod asset_event;
/// 篮子相关事件模块
pub mod basket_event;
/// 指数代币相关事件模块
pub mod index_token_event;

use anchor_lang::prelude::*;
use anchor_lang::event;

/// 篮子指数铸造事件
/// 表示某篮子指数被成功铸造（Mint）时的链上事件，便于追踪指数创建、成交量与算法来源。
#[event]
pub struct BasketIndexMinted {
    /// 篮子ID
    pub basket_id: u64,
    /// 实际成交数量（本次铸造的指数数量）
    pub executed_amount: u64,
    /// 使用的算法名称（如 TWAP、VWAP 等）
    pub algorithm: String,
    /// 铸造时间戳（Unix 时间戳，秒）
    pub timestamp: i64,
}

// 统一 re-export 便于外部直接引用所有事件类型
pub use asset_event::*;
pub use basket_event::*;
pub use index_token_event::*;
