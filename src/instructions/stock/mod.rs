//! # 股票 (Stock) 资产类型指令模块
//!
//! 本模块实现了股票资产的完整功能指令集，包括基础操作、股票特有功能、交易操作、合规功能和高级功能。
//!
//! ## 功能指令集
//!
//! ### 基础操作指令
//! - `mint_stock()` - 增发股票
//! - `burn_stock()` - 销毁股票
//! - `transfer_stock()` - 转账股票
//! - `query_stock()` - 查询股票
//!
//! ### 股票特有功能指令
//! - `dividend_payment()` - 分红支付
//! - `voting_rights()` - 投票权
//! - `corporate_actions()` - 公司行为
//! - `regulatory_compliance()` - 监管合规
//!
//! ### 交易操作指令
//! - `buy_stock()` - 买入股票
//! - `sell_stock()` - 卖出股票
//! - `swap_stock()` - 兑换股票
//! - `quote_stock()` - 报价股票
//!
//! ### 合规功能指令
//! - `kyc_verification()` - KYC验证
//! - `aml_check()` - 反洗钱检查
//! - `trading_hours()` - 交易时间
//! - `circuit_breaker()` - 熔断机制
//!
//! ### 高级功能指令
//! - `limit_orders()` - 限价单
//! - `stop_orders()` - 止损单
//! - `margin_trading()` - 保证金交易
//! - `short_selling()` - 卖空交易
//!
//! ### 批量操作指令
//! - `batch_trade_stock()` - 批量交易股票
//! - `batch_process_stock()` - 批量处理股票
//! - `batch_manage_stock()` - 批量管理股票
//! - `batch_sync_stock()` - 批量同步股票
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

// 股票特有功能指令
pub mod dividend_payment;
pub mod voting_rights;
pub mod corporate_actions;
pub mod regulatory_compliance;

// 交易操作指令
pub mod buy;
pub mod sell;
pub mod swap;
pub mod quote;

// 合规功能指令
pub mod kyc_verification;
pub mod aml_check;
pub mod trading_hours;
pub mod circuit_breaker;

// 高级功能指令
pub mod limit_orders;
pub mod stop_orders;
pub mod margin_trading;
pub mod short_selling;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出股票特有功能指令
pub use dividend_payment::*;
pub use voting_rights::*;
pub use corporate_actions::*;
pub use regulatory_compliance::*;

// 重新导出交易操作指令
pub use buy::*;
pub use sell::*;
pub use swap::*;
pub use quote::*;

// 重新导出合规功能指令
pub use kyc_verification::*;
pub use aml_check::*;
pub use trading_hours::*;
pub use circuit_breaker::*;

// 重新导出高级功能指令
pub use limit_orders::*;
pub use stop_orders::*;
pub use margin_trading::*;
pub use short_selling::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use dividend_payment::{DividendPaymentParams, DividendPayment};
pub use voting_rights::{VotingRightsParams, VotingRights};
pub use corporate_actions::{CorporateActionsParams, CorporateActions};
pub use regulatory_compliance::{RegulatoryComplianceParams, RegulatoryCompliance};
pub use kyc_verification::{KycVerificationParams, KycVerification};
pub use aml_check::{AmlCheckParams, AmlCheck};
pub use trading_hours::{TradingHoursParams, TradingHours};
pub use circuit_breaker::{CircuitBreakerParams, CircuitBreaker};
pub use limit_orders::{LimitOrdersParams, LimitOrders};
pub use stop_orders::{StopOrdersParams, StopOrders};
pub use margin_trading::{MarginTradingParams, MarginTrading};
pub use short_selling::{ShortSellingParams, ShortSelling};
pub use batch::{BatchTradeStockParams, BatchProcessStockParams, BatchManageStockParams, BatchSyncStockParams, BatchStock, BatchOperationType, BatchTradeType, BatchProcessType, BatchManageType, BatchSyncType, BatchOperationResult}; 