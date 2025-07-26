//!
//! token.rs - 代币信息、权重分配、资产类型定义
//!
//! 本文件定义了TokenInfo、WeightAllocation、AssetType等结构体及其实现，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use super::{AssetOperationTrait, AssetBatchOperation, AssetBatchOpType};

/// 代币信息结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct TokenInfo {
    /// 代币 mint 地址
    pub mint: Pubkey,
    /// 代币符号（如 "SOL", "USDC"）
    pub symbol: String,
    /// 代币小数位数（如 9 位 for SOL, 6 位 for USDC）
    pub decimals: u8,
    /// 当前价格（最小单位）
    pub price: u64,
    /// 是否激活用于交易
    pub is_active: bool,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            mint: Pubkey::default(),
            symbol: String::new(),
            decimals: 0,
            price: 0,
            is_active: false,
        }
    }
}

impl TokenInfo {
    /// 构造函数
    pub fn new(
        mint: Pubkey,
        symbol: String,
        decimals: u8,
        price: u64,
        is_active: bool,
    ) -> Result<Self> {
        Ok(Self {
            mint,
            symbol,
            decimals,
            price,
            is_active,
        })
    }
    /// 转为可读金额
    pub fn to_human_readable(&self, amount: u64) -> f64 {
        amount as f64 / 10f64.powi(self.decimals as i32)
    }
    /// 从可读金额转为链上最小单位
    pub fn from_human_readable(&self, amount: f64) -> u64 {
        (amount * 10f64.powi(self.decimals as i32)).round() as u64
    }
    /// 计算市值
    pub fn market_value(&self, amount: u64) -> u64 {
        self.price.saturating_mul(amount)
    }
    /// 预览买入操作，返回预期获得的资产数量（此处为mock实现，实际应接入DEX/AMM报价）
    pub fn preview_buy(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, amount: u64) -> anchor_lang::prelude::Result<u64> {
        // 直接返回输入数量，实际应根据市场价格计算
        Ok(amount)
    }
    /// 预览卖出操作，返回预期获得的资产数量（mock实现）
    pub fn preview_sell(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, amount: u64) -> anchor_lang::prelude::Result<u64> {
        Ok(amount)
    }
    /// 预览兑换操作，返回预期获得的目标资产数量（mock实现）
    pub fn preview_swap(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _target_asset: &TokenInfo, amount: u64) -> anchor_lang::prelude::Result<u64> {
        Ok(amount)
    }
    /// 预览组合操作，返回预期组合后获得的资产数量（mock实现）
    pub fn preview_combine(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _target_asset: &TokenInfo, amount: u64) -> anchor_lang::prelude::Result<u64> {
        Ok(amount)
    }
    /// 预览分割操作，返回预期分割后获得的资产数量（mock实现）
    pub fn preview_split(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, amount: u64) -> anchor_lang::prelude::Result<u64> {
        Ok(amount)
    }
    /// 预览授权操作，返回预期结果（mock实现，返回0）
    pub fn preview_authorize(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _authority: &anchor_lang::prelude::Pubkey) -> anchor_lang::prelude::Result<u64> {
        Ok(0)
    }
    /// 预览冻结操作，返回预期结果（mock实现，返回0）
    pub fn preview_freeze(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo) -> anchor_lang::prelude::Result<u64> {
        Ok(0)
    }
    /// 预览解冻操作，返回预期结果（mock实现，返回0）
    pub fn preview_unfreeze(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo) -> anchor_lang::prelude::Result<u64> {
        Ok(0)
    }
    /// 验证买入操作的有效性（mock实现，始终通过）
    pub fn validate_buy(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _amount: u64) -> anchor_lang::prelude::Result<()> {
        Ok(())
    }
    /// 验证卖出操作的有效性（mock实现，始终通过）
    pub fn validate_sell(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _amount: u64) -> anchor_lang::prelude::Result<()> {
        Ok(())
    }
    /// 验证兑换操作的有效性（mock实现，始终通过）
    pub fn validate_swap(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _target_asset: &TokenInfo, _amount: u64) -> anchor_lang::prelude::Result<()> {
        Ok(())
    }
    /// 验证组合操作的有效性（mock实现，始终通过）
    pub fn validate_combine(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _target_asset: &TokenInfo, _amount: u64) -> anchor_lang::prelude::Result<()> {
        Ok(())
    }
    /// 验证分割操作的有效性（mock实现，始终通过）
    pub fn validate_split(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _amount: u64) -> anchor_lang::prelude::Result<()> {
        Ok(())
    }
    /// 验证授权操作的有效性（mock实现，始终通过）
    pub fn validate_authorize(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo, _authority: &anchor_lang::prelude::Pubkey) -> anchor_lang::prelude::Result<()> {
        Ok(())
    }
    /// 验证冻结操作的有效性（mock实现，始终通过）
    pub fn validate_freeze(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo) -> anchor_lang::prelude::Result<()> {
        Ok(())
    }
    /// 验证解冻操作的有效性（mock实现，始终通过）
    pub fn validate_unfreeze(_ctx: &anchor_lang::prelude::Context<crate::instructions::batch_trade::BatchTrade>, _asset: &TokenInfo) -> anchor_lang::prelude::Result<()> {
        Ok(())
    }
}

impl AssetTrait for TokenInfo {
    /// 获取资产类型名称
    fn asset_type_name(&self) -> &'static str {
        // 返回资产类型名称，便于多资产类型融合
        self.asset_type().asset_type_name()
    }
    /// 是否为可交易资产
    fn is_tradable(&self) -> bool {
        // 判断资产是否可交易，便于统一校验
        self.asset_type().is_tradable()
    }
    /// 获取资产类型枚举
    fn asset_type(&self) -> AssetType {
        // 通过symbol或其他元数据判断资产类型，示例实现：
        match self.symbol.as_str() {
            "USDC" | "USDT" => AssetType::Stablecoin,
            "ETF" => AssetType::ETF,
            "RWA" => AssetType::RWA,
            "STOCK" => AssetType::Stock,
            "INDEX" => AssetType::IndexToken,
            _ => AssetType::Crypto,
        }
    }
    /// mint操作，最小功能单元，逐行注释
    fn mint(&mut self, amount: u64) -> Result<()> {
        // 资产mint操作，增加当前价格（示例，实际应为供应量等）
        self.price = self.price.saturating_add(amount);
        Ok(())
    }
    /// burn操作，最小功能单元，逐行注释
    fn burn(&mut self, amount: u64) -> Result<()> {
        // 资产burn操作，减少当前价格（示例，实际应为供应量等）
        require!(self.price >= amount, crate::error::ErrorCode::InvalidParams);
        self.price -= amount;
        Ok(())
    }
    /// 买入操作，最小功能单元，逐行注释
    fn buy(&mut self, _amount: u64) -> Result<()> {
        // 资产买入操作，实际应对接DEX/AMM，示例直接返回Ok
        Ok(())
    }
    /// 卖出操作，最小功能单元，逐行注释
    fn sell(&mut self, _amount: u64) -> Result<()> {
        // 资产卖出操作，实际应对接DEX/AMM，示例直接返回Ok
        Ok(())
    }
    /// 资产兑换操作，最小功能单元，逐行注释
    fn swap(&mut self, _from_amount: u64, _to_asset: &mut dyn AssetTrait) -> Result<()> {
        // 资产兑换操作，实际应对接DEX/AMM，示例直接返回Ok
        Ok(())
    }
    /// 资产组合操作，最小功能单元，逐行注释
    fn combine(&mut self, _other: &mut dyn AssetTrait, _amount: u64) -> Result<()> {
        // 资产组合操作，实际应对接组合逻辑，示例直接返回Ok
        Ok(())
    }
    /// 资产分割操作，最小功能单元，逐行注释
    fn split(&mut self, _amount: u64) -> Result<()> {
        // 资产分割操作，实际应对接分割逻辑，示例直接返回Ok
        Ok(())
    }
    /// 授权操作，最小功能单元，逐行注释
    fn authorize(&mut self, _authority: Pubkey) -> Result<()> {
        // 资产授权操作，实际应对接权限管理，示例直接返回Ok
        Ok(())
    }
    /// 冻结操作，最小功能单元，逐行注释
    fn freeze(&mut self) -> Result<()> {
        // 资产冻结操作，实际应对接冻结逻辑，示例直接返回Ok
        Ok(())
    }
    /// 解冻操作，最小功能单元，逐行注释
    fn unfreeze(&mut self) -> Result<()> {
        // 资产解冻操作，实际应对接解冻逻辑，示例直接返回Ok
        Ok(())
    }
}

/// 权重分配结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct WeightAllocation {
    /// 代币 mint 地址
    pub token_mint: Pubkey,
    /// 权重（基点）
    pub weight_bps: u64,
}

impl Default for WeightAllocation {
    fn default() -> Self {
        Self {
            token_mint: Pubkey::default(),
            weight_bps: 0,
        }
    }
}

impl WeightAllocation {
    /// 构造函数
    pub fn new(token_mint: Pubkey, weight_bps: u64) -> Result<Self> {
        Ok(Self { token_mint, weight_bps })
    }
    /// 获取权重百分比
    pub fn weight_percentage(&self) -> f64 {
        self.weight_bps as f64 / 10000.0
    }
    /// 设置权重百分比
    pub fn set_weight_percentage(&mut self, percentage: f64) -> Result<()> {
        require!(percentage >= 0.0 && percentage <= 1.0, crate::error::ErrorCode::InvalidParams);
        self.weight_bps = (percentage * 10000.0).round() as u64;
        Ok(())
    }
}

/// 资产类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum AssetType {
    /// 加密货币（如SOL、BTC、ETH等）
    Crypto,
    /// 稳定币（如USDC、USDT等）
    Stablecoin,
    /// 指数代币（如DeFi指数、Layer2指数等）
    IndexToken,
    /// 交易型开放式指数基金（ETF）
    ETF,
    /// 股票（如美股、港股等）
    Stock,
    /// RWA（Real World Asset，现实世界资产，如债券、房产等）
    RWA,
    /// 其他类型
    Other,
}

/// 资产类型trait，所有资产类型必须实现该trait，便于多资产类型融合、扩展、统一操作
pub trait AssetTrait {
    /// 获取资产类型名称
    fn asset_type_name(&self) -> &'static str;
    /// 是否为可交易资产
    fn is_tradable(&self) -> bool;
    /// 获取资产类型枚举
    fn asset_type(&self) -> AssetType;
    /// mint操作，最小功能单元，逐行注释
    fn mint(&mut self, _amount: u64) -> Result<()>;
    /// burn操作，最小功能单元，逐行注释
    fn burn(&mut self, _amount: u64) -> Result<()>;
    /// 买入操作，最小功能单元，逐行注释
    fn buy(&mut self, _amount: u64) -> Result<()>;
    /// 卖出操作，最小功能单元，逐行注释
    fn sell(&mut self, _amount: u64) -> Result<()>;
    /// 资产兑换操作，最小功能单元，逐行注释
    fn swap(&mut self, _from_amount: u64, _to_asset: &mut dyn AssetTrait) -> Result<()>;
    /// 资产组合操作，最小功能单元，逐行注释
    fn combine(&mut self, _other: &mut dyn AssetTrait, _amount: u64) -> Result<()>;
    /// 资产分割操作，最小功能单元，逐行注释
    fn split(&mut self, _amount: u64) -> Result<()>;
    /// 授权操作，最小功能单元，逐行注释
    fn authorize(&mut self, _authority: Pubkey) -> Result<()>;
    /// 冻结操作，最小功能单元，逐行注释
    fn freeze(&mut self) -> Result<()>;
    /// 解冻操作，最小功能单元，逐行注释
    fn unfreeze(&mut self) -> Result<()>;
}

// 为每个AssetType实现AssetTrait，便于统一多资产类型操作和未来扩展
impl AssetTrait for AssetType {
    /// 获取资产类型名称
    fn asset_type_name(&self) -> &'static str {
        match self {
            AssetType::Crypto => "Crypto",
            AssetType::Stablecoin => "Stablecoin",
            AssetType::IndexToken => "IndexToken",
            AssetType::ETF => "ETF",
            AssetType::Stock => "Stock",
            AssetType::RWA => "RWA",
            AssetType::Other => "Other",
        }
    }
    /// 是否为可交易资产
    fn is_tradable(&self) -> bool {
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => true,
            AssetType::Other => false,
        }
    }
    /// 获取资产类型枚举
    fn asset_type(&self) -> AssetType {
        self.clone()
    }
    /// mint操作，最小功能单元，逐行注释
    fn mint(&mut self, _amount: u64) -> Result<()> {
        // 资产类型mint操作，按类型细化
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => Ok(()),
            AssetType::Other => Err(crate::error::ErrorCode::InvalidOperation.into()),
        }
    }
    /// burn操作，最小功能单元，逐行注释
    fn burn(&mut self, _amount: u64) -> Result<()> {
        // 资产类型burn操作，按类型细化
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => Ok(()),
            AssetType::Other => Err(crate::error::ErrorCode::InvalidOperation.into()),
        }
    }
    /// 买入操作，最小功能单元，逐行注释
    fn buy(&mut self, _amount: u64) -> Result<()> {
        // 资产类型买入操作，按类型细化
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => Ok(()),
            AssetType::Other => Err(crate::error::ErrorCode::InvalidOperation.into()),
        }
    }
    /// 卖出操作，最小功能单元，逐行注释
    fn sell(&mut self, _amount: u64) -> Result<()> {
        // 资产类型卖出操作，按类型细化
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => Ok(()),
            AssetType::Other => Err(crate::error::ErrorCode::InvalidOperation.into()),
        }
    }
    /// 资产兑换操作，最小功能单元，逐行注释
    fn swap(&mut self, _from_amount: u64, _to_asset: &mut dyn AssetTrait) -> Result<()> {
        // 资产类型兑换操作，按类型细化
        if self.is_tradable() && _to_asset.is_tradable() {
            Ok(())
        } else {
            Err(crate::error::ErrorCode::InvalidOperation.into())
        }
    }
    /// 资产组合操作，最小功能单元，逐行注释
    fn combine(&mut self, _other: &mut dyn AssetTrait, _amount: u64) -> Result<()> {
        // 资产类型组合操作，按类型细化
        if self.is_tradable() && _other.is_tradable() {
            Ok(())
        } else {
            Err(crate::error::ErrorCode::InvalidOperation.into())
        }
    }
    /// 资产分割操作，最小功能单元，逐行注释
    fn split(&mut self, _amount: u64) -> Result<()> {
        // 资产类型分割操作，按类型细化
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => Ok(()),
            AssetType::Other => Err(crate::error::ErrorCode::InvalidOperation.into()),
        }
    }
    /// 授权操作，最小功能单元，逐行注释
    fn authorize(&mut self, _authority: Pubkey) -> Result<()> {
        // 资产类型授权操作，按类型细化
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => Ok(()),
            AssetType::Other => Err(crate::error::ErrorCode::InvalidOperation.into()),
        }
    }
    /// 冻结操作，最小功能单元，逐行注释
    fn freeze(&mut self) -> Result<()> {
        // 资产类型冻结操作，按类型细化
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => Ok(()),
            AssetType::Other => Err(crate::error::ErrorCode::InvalidOperation.into()),
        }
    }
    /// 解冻操作，最小功能单元，逐行注释
    fn unfreeze(&mut self) -> Result<()> {
        // 资产类型解冻操作，按类型细化
        match self {
            AssetType::Crypto | AssetType::Stablecoin | AssetType::IndexToken | AssetType::ETF | AssetType::Stock | AssetType::RWA => Ok(()),
            AssetType::Other => Err(crate::error::ErrorCode::InvalidOperation.into()),
        }
    }
}

impl AssetOperationTrait for TokenInfo {
    /// 买入操作，TokenInfo本身不直接实现，返回Ok(())
    fn buy(&mut self, _amount: u64) -> Result<()> {
        Ok(())
    }
    /// 卖出操作，TokenInfo本身不直接实现，返回Ok(())
    fn sell(&mut self, _amount: u64) -> Result<()> {
        Ok(())
    }
    /// 资产兑换操作，TokenInfo本身不直接实现，返回Ok(())
    fn swap(&mut self, _from_amount: u64, _to_asset: &mut dyn AssetTrait) -> Result<()> {
        Ok(())
    }
    /// 资产组合操作，TokenInfo本身不直接实现，返回Ok(())
    fn combine(&mut self, _other: &mut dyn AssetTrait, _amount: u64) -> Result<()> {
        Ok(())
    }
    /// 资产分割操作，TokenInfo本身不直接实现，返回Ok(())
    fn split(&mut self, _amount: u64) -> Result<()> {
        Ok(())
    }
    /// 授权操作，TokenInfo本身不直接实现，返回Ok(())
    fn authorize(&mut self, _authority: Pubkey) -> Result<()> {
        Ok(())
    }
    /// 冻结操作，TokenInfo本身不直接实现，返回Ok(())
    fn freeze(&mut self) -> Result<()> {
        Ok(())
    }
    /// 解冻操作，TokenInfo本身不直接实现，返回Ok(())
    fn unfreeze(&mut self) -> Result<()> {
        Ok(())
    }
    /// 批量操作，TokenInfo本身不直接实现，遍历所有批量操作，返回Ok(())
    fn batch_operate(&mut self, _operations: Vec<AssetBatchOperation>) -> Result<()> {
        Ok(())
    }
} 