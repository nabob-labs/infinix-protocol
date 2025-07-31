//! Asset Events
//! 资产相关事件定义，所有资产操作相关事件均在此统一声明，便于链上可观测性和追溯。

use anchor_lang::prelude::*; // Anchor 预导入，包含 #[event]、Pubkey、Result 等

#[event]
/// 资产增发事件
pub struct AssetMinted {
    /// 所属篮子ID
    pub basket_id: u64,
    /// 增发数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产销毁事件
/// 表示某资产被销毁（Burn）时的链上事件，便于追踪资产生命周期。
pub struct AssetBurned {
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
/// 资产买入事件
/// 表示某资产被买入时的链上事件，记录买入价格、买方等信息。
pub struct AssetBought {
    /// 所属篮子ID
    pub basket_id: u64,
    /// 买入数量
    pub amount: u64,
    /// 买入价格（单位：最小计价单位）
    pub price: u64,
    /// 买方地址
    pub buyer: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产卖出事件
/// 表示某资产被卖出时的链上事件，记录卖出价格、卖方等信息。
pub struct AssetSold {
    /// 所属篮子ID
    pub basket_id: u64,
    /// 卖出数量
    pub amount: u64,
    /// 卖出价格（单位：最小计价单位）
    pub price: u64,
    /// 卖方地址
    pub seller: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产兑换事件
/// 表示资产间兑换（Swap）操作的链上事件，记录兑换前后资产及数量。
pub struct AssetSwapped {
    /// 源资产ID
    pub from_asset_id: u64,
    /// 目标资产ID
    pub to_asset_id: u64,
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
/// 资产授权变更事件
/// 表示资产权限变更（如更换管理人）的链上事件。
pub struct AssetAuthorized {
    /// 资产ID
    pub asset_id: u64,
    /// 原权限地址
    pub old_authority: Pubkey,
    /// 新权限地址
    pub new_authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产合并事件
/// 表示两个资产合并操作的链上事件。
pub struct AssetCombined {
    /// 目标资产ID（合并后）
    pub target_asset_id: u64,
    /// 源资产ID（被合并）
    pub source_asset_id: u64,
    /// 合并数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产拆分事件
/// 表示资产拆分操作的链上事件。
pub struct AssetSplit {
    /// 源资产ID（被拆分）
    pub source_asset_id: u64,
    /// 新资产ID（拆分后）
    pub new_asset_id: u64,
    /// 拆分数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产冻结事件
/// 表示资产被冻结时的链上事件。
pub struct AssetFrozen {
    /// 资产ID
    pub asset_id: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产解冻事件
/// 表示资产被解冻时的链上事件。
pub struct AssetUnfrozen {
    /// 资产ID
    pub asset_id: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产批量转移事件
/// 表示资产批量转移操作的链上事件。
pub struct AssetBatchTransferred {
    /// 源资产ID
    pub from_asset_id: u64,
    /// 目标资产ID列表
    pub to_asset_ids: Vec<u64>,
    /// 各目标资产的转移数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产批量兑换事件
/// 表示资产批量兑换操作的链上事件。
pub struct AssetBatchSwapped {
    /// 源资产ID
    pub from_asset_id: u64,
    /// 目标资产ID列表
    pub to_asset_ids: Vec<u64>,
    /// 各源资产的兑换数量
    pub from_amounts: Vec<u64>,
    /// 各目标资产的兑换数量
    pub to_amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产策略交易事件
/// 表示资产通过策略执行交易的链上事件。
pub struct AssetStrategyTraded {
    /// 资产ID
    pub asset_id: u64,
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
/// 资产转移事件
/// 表示资产转移操作的链上事件。
pub struct AssetTransferred {
    /// 源资产ID
    pub from_asset_id: u64,
    /// 目标资产ID
    pub to_asset_id: u64,
    /// 转移数量
    pub amount: u64,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 资产报价事件
/// 表示资产报价查询的链上事件。
pub struct AssetQuoted {
    /// 资产ID
    pub asset_id: u64,
    /// 报价数量
    pub amount: u64,
    /// 报价价格
    pub price: u64,
    /// 报价人
    pub quoter: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}

#[event]
/// 批量资产转移事件
/// 表示批量资产转移操作的链上事件。
pub struct BatchAssetTransferred {
    /// 源资产ID
    pub from_asset_id: u64,
    /// 目标资产ID列表
    pub to_asset_ids: Vec<u64>,
    /// 各目标资产的转移数量
    pub amounts: Vec<u64>,
    /// 操作人
    pub authority: Pubkey,
    /// 事件发生时间戳
    pub timestamp: i64,
}
