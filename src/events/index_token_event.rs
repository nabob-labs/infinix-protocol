//! Index Token Events
//! 指数代币相关事件定义，所有指数代币操作相关事件均在此统一声明，便于链上可观测性和追溯。

use anchor_lang::prelude::*; // Anchor 预导入，包含 #[event]、Pubkey、Result 等

#[event]
/// 指数代币铸造事件
/// 表示指数代币被铸造时的链上事件。
pub struct IndexTokenMinted {
    /// 所属篮子ID
    pub basket_id: u64,
    /// 铸造数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币销毁事件
/// 表示指数代币被销毁时的链上事件。
pub struct IndexTokenBurned {
    /// 所属篮子ID
    pub basket_id: u64,
    /// 销毁数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币买入事件
/// 表示指数代币被买入时的链上事件。
pub struct IndexTokenBought {
    /// 所属篮子ID
    pub basket_id: u64,
    /// 买入数量
    pub amount: u64,
    /// 买入价格
    pub price: u64,
    /// 买方地址
    pub buyer: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币卖出事件
/// 表示指数代币被卖出时的链上事件。
pub struct IndexTokenSold {
    /// 所属篮子ID
    pub basket_id: u64,
    /// 卖出数量
    pub amount: u64,
    /// 卖出价格
    pub price: u64,
    /// 卖方地址
    pub seller: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币兑换事件
/// 表示指数代币间兑换操作的链上事件。
pub struct IndexTokenSwapped {
    /// 源指数代币ID
    pub from_index_token_id: u64,
    /// 目标指数代币ID
    pub to_index_token_id: u64,
    /// 兑换前数量
    pub from_amount: u64,
    /// 兑换后数量
    pub to_amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币授权变更事件
/// 表示指数代币权限变更（如更换管理人）的链上事件。
pub struct IndexTokenAuthorized {
    /// 指数代币ID
    pub index_token_id: u64,
    /// 原权限地址
    pub old_authority: Pubkey,
    /// 新权限地址
    pub new_authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币合并事件
/// 表示两个指数代币合并操作的链上事件。
pub struct IndexTokenCombined {
    /// 目标指数代币ID（合并后）
    pub target_index_token_id: u64,
    /// 源指数代币ID（被合并）
    pub source_index_token_id: u64,
    /// 合并数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币拆分事件
/// 表示指数代币拆分操作的链上事件。
pub struct IndexTokenSplit {
    /// 源指数代币ID（被拆分）
    pub source_index_token_id: u64,
    /// 新指数代币ID（拆分后）
    pub new_index_token_id: u64,
    /// 拆分数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币冻结事件
/// 表示指数代币被冻结时的链上事件。
pub struct IndexTokenFrozen {
    /// 指数代币ID
    pub index_token_id: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币解冻事件
/// 表示指数代币被解冻时的链上事件。
pub struct IndexTokenUnfrozen {
    /// 指数代币ID
    pub index_token_id: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币批量转移事件
/// 表示指数代币批量转移操作的链上事件。
pub struct IndexTokenBatchTransferred {
    /// 源指数代币ID
    pub from_index_token_id: u64,
    /// 目标指数代币ID列表
    pub to_index_token_ids: Vec<u64>,
    /// 各目标指数代币的转移数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币批量申购事件
/// 表示指数代币批量申购操作的链上事件。
pub struct IndexTokenBatchSubscribed {
    /// 指数代币ID
    pub index_token_id: u64,
    /// 各申购数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币批量赎回事件
/// 表示指数代币批量赎回操作的链上事件。
pub struct IndexTokenBatchRedeemed {
    /// 指数代币ID
    pub index_token_id: u64,
    /// 各赎回数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币策略申购事件
/// 表示指数代币通过策略执行申购的链上事件。
pub struct IndexTokenStrategySubscribed {
    /// 指数代币ID
    pub index_token_id: u64,
    /// 策略名称
    pub strategy: String,
    /// 策略参数（序列化字节）
    pub params: Vec<u8>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币策略赎回事件
/// 表示指数代币通过策略执行赎回的链上事件。
pub struct IndexTokenStrategyRedeemed {
    /// 指数代币ID
    pub index_token_id: u64,
    /// 策略名称
    pub strategy: String,
    /// 策略参数（序列化字节）
    pub params: Vec<u8>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币批量合并事件
/// 表示多个指数代币合并操作的链上事件。
pub struct IndexTokenBatchCombined {
    /// 目标指数代币ID（合并后）
    pub target_index_token_id: u64,
    /// 源指数代币ID列表（被合并）
    pub source_index_token_ids: Vec<u64>,
    /// 各合并数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 指数代币批量拆分事件
/// 表示指数代币批量拆分操作的链上事件。
pub struct IndexTokenBatchSplit {
    /// 源指数代币ID（被拆分）
    pub source_index_token_id: u64,
    /// 新指数代币ID列表（拆分后）
    pub new_index_token_ids: Vec<u64>,
    /// 各拆分数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}
