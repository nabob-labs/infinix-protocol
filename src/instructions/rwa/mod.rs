//! # 现实世界资产 (RWA) 资产类型指令模块
//!
//! 本模块实现了现实世界资产的完整功能指令集，包括基础操作、RWA特有功能、交易操作、合规功能和高级功能。
//!
//! ## 功能指令集
//!
//! ### 基础操作指令
//! - `mint_rwa()` - 增发RWA
//! - `burn_rwa()` - 销毁RWA
//! - `transfer_rwa()` - 转账RWA
//! - `query_rwa()` - 查询RWA
//!
//! ### RWA特有功能指令
//! - `asset_tokenization()` - 资产代币化
//! - `legal_compliance()` - 法律合规
//! - `custody_management()` - 托管管理
//! - `valuation_verification()` - 估值验证
//!
//! ### 交易操作指令
//! - `buy_rwa()` - 买入RWA
//! - `sell_rwa()` - 卖出RWA
//! - `swap_rwa()` - 兑换RWA
//! - `quote_rwa()` - 报价RWA
//!
//! ### 合规功能指令
//! - `regulatory_reporting()` - 监管报告
//! - `audit_trail()` - 审计追踪
//! - `legal_documentation()` - 法律文件
//! - `insurance_coverage()` - 保险覆盖
//!
//! ### 高级功能指令
//! - `fractional_ownership()` - 部分所有权
//! - `revenue_sharing()` - 收益分享
//! - `governance_rights()` - 治理权
//! - `exit_strategy()` - 退出策略
//!
//! ### 批量操作指令
//! - `batch_trade_rwa()` - 批量交易RWA
//! - `batch_process_rwa()` - 批量处理RWA
//! - `batch_manage_rwa()` - 批量管理RWA
//! - `batch_sync_rwa()` - 批量同步RWA
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

// RWA特有功能指令
pub mod asset_tokenization;
pub mod legal_compliance;
pub mod custody_management;
pub mod valuation_verification;

// 交易操作指令
pub mod buy;
pub mod sell;
pub mod swap;
pub mod quote;

// 合规功能指令
pub mod regulatory_reporting;
pub mod audit_trail;
pub mod legal_documentation;
pub mod insurance_coverage;

// 高级功能指令
pub mod fractional_ownership;
pub mod revenue_sharing;
pub mod governance_rights;
pub mod exit_strategy;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出RWA特有功能指令
pub use asset_tokenization::*;
pub use legal_compliance::*;
pub use custody_management::*;
pub use valuation_verification::*;

// 重新导出交易操作指令
pub use buy::*;
pub use sell::*;
pub use swap::*;
pub use quote::*;

// 重新导出合规功能指令
pub use regulatory_reporting::*;
pub use audit_trail::*;
pub use legal_documentation::*;
pub use insurance_coverage::*;

// 重新导出高级功能指令
pub use fractional_ownership::*;
pub use revenue_sharing::*;
pub use governance_rights::*;
pub use exit_strategy::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use asset_tokenization::{AssetTokenizationParams, AssetTokenization};
pub use legal_compliance::{LegalComplianceParams, LegalCompliance};
pub use custody_management::{CustodyManagementParams, CustodyManagement};
pub use valuation_verification::{ValuationVerificationParams, ValuationVerification};
pub use regulatory_reporting::{RegulatoryReportingParams, RegulatoryReporting};
pub use audit_trail::{AuditTrailParams, AuditTrail};
pub use legal_documentation::{LegalDocumentationParams, LegalDocumentation};
pub use insurance_coverage::{InsuranceCoverageParams, InsuranceCoverage};
pub use fractional_ownership::{FractionalOwnershipParams, FractionalOwnership};
pub use revenue_sharing::{RevenueSharingParams, RevenueSharing};
pub use governance_rights::{GovernanceRightsParams, GovernanceRights};
pub use exit_strategy::{ExitStrategyParams, ExitStrategy};
pub use batch::{BatchTradeRwaParams, BatchProcessRwaParams, BatchManageRwaParams, BatchSyncRwaParams, BatchRwa, BatchOperationType, BatchTradeType, BatchProcessType, BatchManageType, BatchSyncType, BatchOperationResult}; 