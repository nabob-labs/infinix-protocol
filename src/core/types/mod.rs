pub mod trade;
pub mod algo;
pub mod strategy;
pub mod oracle;
pub mod dex;
pub mod risk;
pub mod market;
pub mod token;
pub mod validatable;

pub use trade::*;
pub use algo::*;
pub use strategy::*;
pub use oracle::*;
pub use dex::*;
pub use risk::*;
pub use market::*;
pub use token::AssetTrait;
pub use token::AssetType;
pub use token::TokenInfo;
pub use token::WeightAllocation;
pub use validatable::*;

/// AssetTrait - 资产类型trait，所有资产类型必须实现该trait，便于多资产类型融合、扩展、统一操作
pub trait AssetTrait {
    /// 获取资产类型名称，返回静态字符串
    fn asset_type_name(&self) -> &'static str;
    /// 是否为可交易资产，返回布尔值
    fn is_tradable(&self) -> bool;
    /// 获取资产类型枚举，返回AssetType
    fn asset_type(&self) -> AssetType;
}

/// AssetOperationTrait - 统一所有资产操作trait接口，所有资产类型均需实现，便于最小功能单元化、可插拔、可扩展
pub trait AssetOperationTrait {
    /// 买入操作，amount为买入数量
    fn buy(&mut self, amount: u64) -> Result<()>;
    /// 卖出操作，amount为卖出数量
    fn sell(&mut self, amount: u64) -> Result<()>;
    /// 资产兑换操作，from_amount为兑换数量，to_asset为目标资产
    fn swap(&mut self, from_amount: u64, to_asset: &mut dyn AssetTrait) -> Result<()>;
    /// 资产组合操作，other为被组合资产，amount为组合数量
    fn combine(&mut self, other: &mut dyn AssetTrait, amount: u64) -> Result<()>;
    /// 资产分割操作，amount为分割数量
    fn split(&mut self, amount: u64) -> Result<()>;
    /// 授权操作，authority为新权限账户
    fn authorize(&mut self, authority: Pubkey) -> Result<()>;
    /// 冻结操作
    fn freeze(&mut self) -> Result<()>;
    /// 解冻操作
    fn unfreeze(&mut self) -> Result<()>;
    /// 批量操作，operations为批量操作参数集合
    fn batch_operate(&mut self, operations: Vec<AssetBatchOperation>) -> Result<()>;
}

/// AssetBatchOperation - 批量操作参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct AssetBatchOperation {
    /// 操作类型，见AssetBatchOpType
    pub op_type: AssetBatchOpType,
    /// 操作数量
    pub amount: u64,
    /// 目标资产（可选）
    pub target: Option<Pubkey>,
}

/// AssetBatchOpType - 批量操作类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum AssetBatchOpType {
    /// 买入
    Buy,
    /// 卖出
    Sell,
    /// 兑换
    Swap,
    /// 组合
    Combine,
    /// 分割
    Split,
    /// 授权
    Authorize,
    /// 冻结
    Freeze,
    /// 解冻
    Unfreeze,
}

/// DexAdapterTrait - 统一所有DEX/AMM外部集成trait接口，所有DEX/AMM适配器均需实现，便于最小功能单元化、可插拔、可扩展
pub trait DexAdapterTrait {
    /// 查询支持的资产类型，返回AssetType集合
    fn supported_assets(&self) -> Vec<AssetType>;
    /// 查询DEX/AMM名称，返回静态字符串
    fn name(&self) -> &'static str;
    /// 查询DEX/AMM版本，返回静态字符串
    fn version(&self) -> &'static str;
    /// 查询是否支持某资产，asset为资产类型
    fn is_asset_supported(&self, asset: &AssetType) -> bool;
    /// 查询流动性，asset为资产类型，返回流动性数量
    fn liquidity(&self, asset: &AssetType) -> Result<u64>;
    /// 查询价格，asset为资产类型，返回价格
    fn price(&self, asset: &AssetType) -> Result<u64>;
    /// 下单（买入），asset为资产类型，amount为买入数量，返回实际成交数量
    fn buy(&mut self, asset: &AssetType, amount: u64) -> Result<u64>;
    /// 下单（卖出），asset为资产类型，amount为卖出数量，返回实际成交数量
    fn sell(&mut self, asset: &AssetType, amount: u64) -> Result<u64>;
    /// 资产兑换，from为源资产，to为目标资产，amount为兑换数量，返回实际兑换数量
    fn swap(&mut self, from: &AssetType, to: &AssetType, amount: u64) -> Result<u64>;
}

/// OracleAdapterTrait - 统一所有预言机外部集成trait接口，所有预言机适配器均需实现，便于最小功能单元化、可插拔、可扩展
pub trait OracleAdapterTrait {
    /// 查询支持的资产类型，返回AssetType集合
    fn supported_assets(&self) -> Vec<AssetType>;
    /// 查询预言机名称，返回静态字符串
    fn name(&self) -> &'static str;
    /// 查询预言机版本，返回静态字符串
    fn version(&self) -> &'static str;
    /// 查询是否支持某资产，asset为资产类型
    fn is_asset_supported(&self, asset: &AssetType) -> bool;
    /// 查询价格，asset为资产类型，返回价格
    fn price(&self, asset: &AssetType) -> Result<u64>;
    /// 查询价格精度，asset为资产类型，返回精度
    fn price_precision(&self, asset: &AssetType) -> Result<u64>;
} 