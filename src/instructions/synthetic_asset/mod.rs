//! Synthetic Asset (合成资产) 资产类型指令模块
//! 
//! 本模块提供Synthetic Asset资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - 合成功能：创建合成资产、抵押、清算、预言机更新
//! - 交易操作：交易、对冲、套利、杠杆
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给SyntheticAssetService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// 合成功能指令
pub mod create_synthetic;
pub mod collateralize;
pub mod liquidate;
pub mod oracle_update;

// 交易操作指令
pub mod trade_synthetic;
pub mod hedge_synthetic;
pub mod arbitrage_synthetic;
pub mod leverage_synthetic;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出合成功能指令
pub use create_synthetic::*;
pub use collateralize::*;
pub use liquidate::*;
pub use oracle_update::*;

// 重新导出交易操作指令
pub use trade_synthetic::*;
pub use hedge_synthetic::*;
pub use arbitrage_synthetic::*;
pub use leverage_synthetic::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintSyntheticParams, MintSynthetic};
pub use burn::{BurnSyntheticParams, BurnSynthetic};
pub use transfer::{TransferSyntheticParams, TransferSynthetic};
pub use query::{QuerySyntheticParams, QuerySynthetic};
pub use create_synthetic::{CreateSyntheticParams, CreateSynthetic};
pub use collateralize::{CollateralizeParams, Collateralize};
pub use liquidate::{LiquidateParams, Liquidate};
pub use oracle_update::{OracleUpdateParams, OracleUpdate};
pub use trade_synthetic::{TradeSyntheticParams, TradeSynthetic};
pub use hedge_synthetic::{HedgeSyntheticParams, HedgeSynthetic};
pub use arbitrage_synthetic::{ArbitrageSyntheticParams, ArbitrageSynthetic};
pub use leverage_synthetic::{LeverageSyntheticParams, LeverageSynthetic};
pub use batch::{BatchTradeSyntheticParams, BatchProcessSyntheticParams, BatchManageSyntheticParams, BatchSyncSyntheticParams, BatchSynthetic}; 