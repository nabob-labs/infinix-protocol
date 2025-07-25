//! Basket Events
//! 篮子相关事件定义，所有篮子操作相关事件均在此统一声明，便于链上可观测性和追溯。

use anchor_lang::prelude::*; // Anchor 预导入，包含 #[event]、Pubkey、Result 等

#[event]
/// 篮子再平衡事件
/// 表示篮子资产权重发生再平衡时的链上事件。
pub struct BasketRebalanced {
    /// 篮子ID
    pub basket_id: u64,
    /// 新权重列表
    pub new_weights: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子暂停事件
/// 表示篮子被暂停时的链上事件。
pub struct BasketPaused {
    /// 篮子ID
    pub basket_id: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子恢复事件
/// 表示暂停的篮子被恢复时的链上事件。
pub struct BasketResumed {
    /// 篮子ID
    pub basket_id: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子买入事件
/// 表示篮子被买入时的链上事件。
pub struct BasketBought {
    /// 篮子ID
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
/// 篮子卖出事件
/// 表示篮子被卖出时的链上事件。
pub struct BasketSold {
    /// 篮子ID
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
/// 篮子兑换事件
/// 表示篮子间兑换操作的链上事件。
pub struct BasketSwapped {
    /// 源篮子ID
    pub from_basket_id: u64,
    /// 目标篮子ID
    pub to_basket_id: u64,
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
/// 篮子授权变更事件
/// 表示篮子权限变更（如更换管理人）的链上事件。
pub struct BasketAuthorized {
    /// 篮子ID
    pub basket_id: u64,
    /// 原权限地址
    pub old_authority: Pubkey,
    /// 新权限地址
    pub new_authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子合并事件
/// 表示两个篮子合并操作的链上事件。
pub struct BasketCombined {
    /// 目标篮子ID（合并后）
    pub target_basket_id: u64,
    /// 源篮子ID（被合并）
    pub source_basket_id: u64,
    /// 合并数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子拆分事件
/// 表示篮子拆分操作的链上事件。
pub struct BasketSplit {
    /// 源篮子ID（被拆分）
    pub source_basket_id: u64,
    /// 新篮子ID（拆分后）
    pub new_basket_id: u64,
    /// 拆分数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子冻结事件
/// 表示篮子被冻结时的链上事件。
pub struct BasketFrozen {
    /// 篮子ID
    pub basket_id: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子解冻事件
/// 表示篮子被解冻时的链上事件。
pub struct BasketUnfrozen {
    /// 篮子ID
    pub basket_id: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子批量转移事件
/// 表示篮子批量转移操作的链上事件。
pub struct BasketBatchTransferred {
    /// 源篮子ID
    pub from_basket_id: u64,
    /// 目标篮子ID列表
    pub to_basket_ids: Vec<u64>,
    /// 各目标篮子的转移数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子批量再平衡事件
/// 表示篮子批量再平衡操作的链上事件。
pub struct BasketBatchRebalanced {
    /// 篮子ID
    pub basket_id: u64,
    /// 新权重列表
    pub new_weights: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子策略再平衡事件
/// 表示篮子通过策略执行再平衡的链上事件。
pub struct BasketStrategyRebalanced {
    /// 篮子ID
    pub basket_id: u64,
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
/// 篮子批量申购事件
/// 表示篮子批量申购操作的链上事件。
pub struct BasketBatchSubscribed {
    /// 篮子ID
    pub basket_id: u64,
    /// 各申购数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子批量赎回事件
/// 表示篮子批量赎回操作的链上事件。
pub struct BasketBatchRedeemed {
    /// 篮子ID
    pub basket_id: u64,
    /// 各赎回数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子批量合并事件
/// 表示多个篮子合并操作的链上事件。
pub struct BasketBatchCombined {
    /// 目标篮子ID（合并后）
    pub target_basket_id: u64,
    /// 源篮子ID列表（被合并）
    pub source_basket_ids: Vec<u64>,
    /// 各合并数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 篮子批量拆分事件
/// 表示篮子批量拆分操作的链上事件。
pub struct BasketBatchSplit {
    /// 源篮子ID（被拆分）
    pub source_basket_id: u64,
    /// 新篮子ID列表（拆分后）
    pub new_basket_ids: Vec<u64>,
    /// 各拆分数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}
