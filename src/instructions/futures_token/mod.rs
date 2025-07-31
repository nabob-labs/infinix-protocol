//! Futures Token (期货代币) 资产类型指令模块
//! 
//! 本模块提供Futures Token资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - 期货功能：创建期货、结算、保证金催缴、清算
//! - 高级功能：期货定价、基差交易、日历价差、蝶式价差
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给FuturesTokenService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// 期货功能指令
pub mod create_futures;
pub mod settle_futures;
pub mod margin_call;
pub mod liquidation;

// 高级功能指令
pub mod futures_pricing;
pub mod basis_trading;
pub mod calendar_spread;
pub mod butterfly_spread;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出期货功能指令
pub use create_futures::*;
pub use settle_futures::*;
pub use margin_call::*;
pub use liquidation::*;

// 重新导出高级功能指令
pub use futures_pricing::*;
pub use basis_trading::*;
pub use calendar_spread::*;
pub use butterfly_spread::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintFuturesParams, MintFutures};
pub use burn::{BurnFuturesParams, BurnFutures};
pub use transfer::{TransferFuturesParams, TransferFutures};
pub use query::{QueryFuturesParams, QueryFutures};
pub use create_futures::{CreateFuturesParams, CreateFutures};
pub use settle_futures::{SettleFuturesParams, SettleFutures};
pub use margin_call::{MarginCallParams, MarginCall};
pub use liquidation::{LiquidationParams, Liquidation};
pub use futures_pricing::{FuturesPricingParams, FuturesPricing};
pub use basis_trading::{BasisTradingParams, BasisTrading};
pub use calendar_spread::{CalendarSpreadParams, CalendarSpread};
pub use butterfly_spread::{ButterflySpreadParams, ButterflySpread};
pub use batch::{BatchTradeFuturesParams, BatchProcessFuturesParams, BatchManageFuturesParams, BatchSyncFuturesParams, BatchFutures}; 