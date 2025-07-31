//! Margin Token (保证金代币) 资产类型指令模块
//! 
//! 本模块提供Margin Token资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - 保证金功能：借入、偿还、清算、利息累积
//! - 高级功能：保证金比率、抵押品比率、风险管理、自动清算
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给MarginTokenService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// 保证金功能指令
pub mod borrow_margin;
pub mod repay_margin;
pub mod liquidate_margin;
pub mod interest_accrual;

// 高级功能指令
pub mod margin_ratio;
pub mod collateral_ratio;
pub mod risk_management;
pub mod auto_liquidation;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出保证金功能指令
pub use borrow_margin::*;
pub use repay_margin::*;
pub use liquidate_margin::*;
pub use interest_accrual::*;

// 重新导出高级功能指令
pub use margin_ratio::*;
pub use collateral_ratio::*;
pub use risk_management::*;
pub use auto_liquidation::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintMarginParams, MintMargin};
pub use burn::{BurnMarginParams, BurnMargin};
pub use transfer::{TransferMarginParams, TransferMargin};
pub use query::{QueryMarginParams, QueryMargin};
pub use borrow_margin::{BorrowMarginParams, BorrowMargin};
pub use repay_margin::{RepayMarginParams, RepayMargin};
pub use liquidate_margin::{LiquidateMarginParams, LiquidateMargin};
pub use interest_accrual::{InterestAccrualParams, InterestAccrual};
pub use margin_ratio::{MarginRatioParams, MarginRatio};
pub use collateral_ratio::{CollateralRatioParams, CollateralRatio};
pub use risk_management::{RiskManagementParams, RiskManagement};
pub use auto_liquidation::{AutoLiquidationParams, AutoLiquidation};
pub use batch::{BatchTradeMarginParams, BatchProcessMarginParams, BatchManageMarginParams, BatchSyncMarginParams, BatchMargin}; 