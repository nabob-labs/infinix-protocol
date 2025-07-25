use anchor_lang::prelude::*; // Anchor预导入，包含Result、Context、Pubkey等
use crate::core::events::OracleEvent; // 统一Oracle事件类型
use crate::core::errors::OracleError; // 统一Oracle错误类型

// ========================= Oracle 统一 Trait 定义 =========================
// 本模块定义所有 Oracle 适配器的统一 trait、参数、结果、错误类型，
// 每个 trait、struct、enum、参数、用途、边界、Anchor 相关点均有详细注释。
// - 设计意图：极致可插拔、最小功能单元、统一接口、Anchor集成友好、可观测性、可维护性、可审计性

/// 预言机适配器统一Trait接口
/// - 统一所有链上预言机的价格、TWAP、VWAP等查询操作
/// - 便于多预言机集成、策略复用、测试模拟
pub trait OracleAdapter: Send + Sync {
    /// 获取适配器名称
    fn name(&self) -> &'static str;
    /// 获取现价
    /// - ctx: Anchor上下文，包含账户信息
    /// - params: 现价查询参数
    /// - 返回：现价查询结果（PriceResult结构体）
    /// - 设计意图：所有Oracle适配器必须实现，便于统一聚合、扩展、测试
    fn get_price(&self, ctx: Context<GetPrice>, params: PriceParams) -> Result<PriceResult>;
    /// 获取TWAP
    /// - ctx: Anchor上下文
    /// - params: TWAP查询参数
    /// - 返回：TWAP查询结果（TwapResult结构体）
    fn get_twap(&self, ctx: Context<GetTwap>, params: TwapParams) -> Result<TwapResult>;
    /// 获取VWAP
    /// - ctx: Anchor上下文
    /// - params: VWAP查询参数
    /// - 返回：VWAP查询结果（VwapResult结构体）
    fn get_vwap(&self, ctx: Context<GetVwap>, params: VwapParams) -> Result<VwapResult>;
    /// 触发事件
    fn emit_event(&self, event: OracleEvent);
    /// 获取最近一次错误
    fn last_error(&self) -> Option<OracleError> { None }
}

/// 现价查询参数结构体
/// - 描述一次get_price操作的所有输入参数
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct PriceParams {
    /// 基础资产mint
    pub base_mint: Pubkey,
    /// 报价资产mint
    pub quote_mint: Pubkey,
}

/// 现价查询结果结构体
/// - 记录get_price操作的价格和更新时间
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct PriceResult {
    /// 价格
    pub price: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
}

/// TWAP查询参数结构体
/// - 描述一次get_twap操作的所有输入参数
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct TwapParams {
    /// 基础资产mint
    pub base_mint: Pubkey,
    /// 报价资产mint
    pub quote_mint: Pubkey,
    /// 时间区间（秒）
    pub interval: u64,
}

/// TWAP查询结果结构体
/// - 记录get_twap操作的TWAP和更新时间
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct TwapResult {
    /// TWAP数值
    pub twap: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
}

/// VWAP查询参数结构体
/// - 描述一次get_vwap操作的所有输入参数
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct VwapParams {
    /// 基础资产mint
    pub base_mint: Pubkey,
    /// 报价资产mint
    pub quote_mint: Pubkey,
    /// 时间区间（秒）
    pub interval: u64,
}

/// VWAP查询结果结构体
/// - 记录get_vwap操作的VWAP和更新时间
#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct VwapResult {
    /// VWAP数值
    pub vwap: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
}

/// 预言机适配器错误类型
/// - 统一所有Oracle适配器的错误类型，便于上层统一处理
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Eq)]
pub enum OracleError {
    /// 账户无效
    InvalidAccount,
    /// 无数据
    NoData,
    /// 数据过期
    StaleData,
    /// 其他错误，附带错误信息
    Other(String),
}

/// MockOracleAdapter为OracleAdapter trait的测试实现，所有方法返回可控的测试数据，便于单元测试和集成测试。
pub struct MockOracleAdapter;

impl OracleAdapter for MockOracleAdapter {
    /// 返回适配器名称
    fn name(&self) -> &'static str {
        "MockOracleAdapter"
    }
    /// 模拟get_price操作，返回固定价格
    fn get_price(&self, _ctx: Context<GetPrice>, _params: PriceParams) -> Result<PriceResult> {
        Ok(PriceResult {
            price: 1_000_000,
            last_updated: 0,
        })
    }
    /// 模拟get_twap操作，返回固定TWAP
    fn get_twap(&self, _ctx: Context<GetTwap>, _params: TwapParams) -> Result<TwapResult> {
        Ok(TwapResult {
            twap: 1_000_000,
            last_updated: 0,
        })
    }
    /// 模拟get_vwap操作，返回固定VWAP
    fn get_vwap(&self, _ctx: Context<GetVwap>, _params: VwapParams) -> Result<VwapResult> {
        Ok(VwapResult {
            vwap: 1_000_000,
            last_updated: 0,
        })
    }
    /// 模拟事件触发（无实际效果）
    fn emit_event(&self, _event: OracleEvent) {
        // Mock implementation
    }
}

/// MockOracleAdapter 是 OracleAdapter trait 的测试实现，所有方法返回可控的测试数据，便于单元测试和集成测试。
