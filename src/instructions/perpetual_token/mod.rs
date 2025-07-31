//! Perpetual Token (永续合约代币) 资产类型指令模块
//! 
//! 本模块提供Perpetual Token资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - 永续功能：开仓、平仓、资金费率、清算
//! - 高级功能：杠杆交易、全仓保证金、逐仓保证金、仓位管理
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给PerpetualTokenService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// 永续功能指令
pub mod open_position;
pub mod close_position;
pub mod funding_rate;
pub mod liquidation;

// 高级功能指令
pub mod leverage_trading;
pub mod cross_margin;
pub mod isolated_margin;
pub mod position_sizing;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出永续功能指令
pub use open_position::*;
pub use close_position::*;
pub use funding_rate::*;
pub use liquidation::*;

// 重新导出高级功能指令
pub use leverage_trading::*;
pub use cross_margin::*;
pub use isolated_margin::*;
pub use position_sizing::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintPerpetualParams, MintPerpetual};
pub use burn::{BurnPerpetualParams, BurnPerpetual};
pub use transfer::{TransferPerpetualParams, TransferPerpetual};
pub use query::{QueryPerpetualParams, QueryPerpetual};
pub use open_position::{OpenPositionParams, OpenPosition};
pub use close_position::{ClosePositionParams, ClosePosition};
pub use funding_rate::{FundingRateParams, FundingRate};
pub use liquidation::{LiquidationParams, Liquidation};
pub use leverage_trading::{LeverageTradingParams, LeverageTrading};
pub use cross_margin::{CrossMarginParams, CrossMargin};
pub use isolated_margin::{IsolatedMarginParams, IsolatedMargin};
pub use position_sizing::{PositionSizingParams, PositionSizing};
pub use batch::{BatchTradePerpetualParams, BatchProcessPerpetualParams, BatchManagePerpetualParams, BatchSyncPerpetualParams, BatchPerpetual}; 