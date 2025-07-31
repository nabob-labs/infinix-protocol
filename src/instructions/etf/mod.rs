//!
//! # ETF (Exchange Traded Fund) 资产类型指令模块
//!
//! 本模块实现了ETF资产的完整功能指令集，包括基础操作、ETF特有功能、交易操作、管理功能和高级功能。
//!
//! ## 功能指令集
//!
//! ### 基础操作指令
//! - `mint_etf()` - 增发ETF
//! - `burn_etf()` - 销毁ETF
//! - `transfer_etf()` - 转账ETF
//! - `query_etf()` - 查询ETF
//!
//! ### ETF特有功能指令
//! - `subscribe_etf()` - 申购ETF
//! - `redeem_etf()` - 赎回ETF
//! - `nav_calculation()` - 净值计算
//! - `creation_redemption()` - 创建赎回
//!
//! ### 交易操作指令
//! - `buy_etf()` - 买入ETF
//! - `sell_etf()` - 卖出ETF
//! - `swap_etf()` - 兑换ETF
//! - `quote_etf()` - 报价ETF
//!
//! ### 管理功能指令
//! - `expense_ratio()` - 费用率管理
//! - `tracking_error()` - 跟踪误差管理
//! - `liquidity_provision()` - 流动性提供
//! - `market_making()` - 做市商机制
//!
//! ### 高级功能指令
//! - `in_kind_creation()` - 实物申购
//! - `cash_creation()` - 现金申购
//! - `arbitrage_monitoring()` - 套利监控
//! - `premium_discount()` - 溢价折价管理
//!
//! ### 批量操作指令
//! - `batch_trade_etf()` - 批量交易ETF
//! - `batch_process_etf()` - 批量处理ETF
//! - `batch_manage_etf()` - 批量管理ETF
//! - `batch_sync_etf()` - 批量同步ETF
//!
//! ## 设计原则
//!
//! - **最小功能单元**: 每个指令功能单一，职责明确
//! - **细粒度设计**: 支持灵活组合和扩展
//! - **类型安全**: 严格的类型检查和边界验证
//! - **性能优化**: 高效的批量操作和算法执行
//! - **合规性**: 符合金融监管和审计标准
//!
//! ## 架构特点
//!
//! - **模块化设计**: 清晰的模块分离和接口定义
//! - **服务层抽象**: 核心业务逻辑委托给服务层
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//! - **权限控制**: 细粒度的权限验证和管理

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// ETF特有功能指令
pub mod subscribe;
pub mod redeem;
pub mod nav_calculation;
pub mod creation_redemption;

// 交易操作指令
pub mod buy;
pub mod sell;
pub mod swap;
pub mod quote;

// 管理功能指令
pub mod expense_ratio;
pub mod tracking_error;
pub mod liquidity_provision;
pub mod market_making;


// 高级功能指令
pub mod in_kind_creation;
pub mod cash_creation;
pub mod arbitrage_monitoring;
pub mod premium_discount;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出ETF特有功能指令
pub use subscribe::*;
pub use redeem::*;
pub use nav_calculation::*;
pub use creation_redemption::*;

// 重新导出交易操作指令
pub use buy::*;
pub use sell::*;
pub use swap::*;
pub use quote::*;

// 重新导出管理功能指令
pub use expense_ratio::*;
pub use tracking_error::*;
pub use liquidity_provision::*;
pub use market_making::*;

// 重新导出高级功能指令
pub use in_kind_creation::*;
pub use cash_creation::*;
pub use arbitrage_monitoring::*;
pub use premium_discount::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use subscribe::{SubscribeEtfParams, SubscribeEtf};
pub use redeem::{RedeemEtfParams, RedeemEtf};
pub use nav_calculation::{NavCalculationParams, NavCalculation};
pub use creation_redemption::{CreationRedemptionParams, CreationRedemption};
pub use expense_ratio::{ExpenseRatioParams, ExpenseRatio};
pub use tracking_error::{TrackingErrorParams, TrackingError};
pub use liquidity_provision::{LiquidityProvisionParams, LiquidityProvision};
pub use market_making::{MarketMakingParams, MarketMaking};
pub use in_kind_creation::{InKindCreationParams, InKindCreation};
pub use cash_creation::{CashCreationParams, CashCreation};
pub use arbitrage_monitoring::{ArbitrageMonitoringParams, ArbitrageMonitoring};
pub use premium_discount::{PremiumDiscountParams, PremiumDiscount};
pub use batch::{BatchTradeEtfParams, BatchProcessEtfParams, BatchManageEtfParams, BatchSyncEtfParams, BatchEtf, BatchOperationType, BatchTradeType, BatchProcessType, BatchManageType, BatchSyncType, BatchOperationResult}; 