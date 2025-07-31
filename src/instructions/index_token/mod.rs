//!
//! # 指数代币 (IndexToken) 资产类型指令模块
//!
//! 本模块实现了指数代币资产的完整功能指令集，包括基础操作、指数管理、交易操作、高级功能和策略指令。
//!
//! ## 功能指令集
//!
//! ### 基础操作指令
//! - `mint_index_token()` - 增发指数代币
//! - `burn_index_token()` - 销毁指数代币
//! - `transfer_index_token()` - 转账指数代币
//! - `query_index_token()` - 查询指数代币
//!
//! ### 指数管理指令
//! - `rebalance_index()` - 指数再平衡
//! - `weight_adjustment()` - 权重调整
//! - `constituent_update()` - 成分股更新
//! - `performance_tracking()` - 表现追踪
//!
//! ### 交易操作指令
//! - `buy_index_token()` - 买入指数代币
//! - `sell_index_token()` - 卖出指数代币
//! - `swap_index_token()` - 兑换指数代币
//! - `quote_index_token()` - 报价指数代币
//!
//! ### 高级功能指令
//! - `dividend_distribution()` - 分红分配
//! - `voting_rights()` - 投票权管理
//! - `governance()` - 治理功能
//! - `fee_management()` - 费用管理
//!
//! ### 策略指令
//! - `dynamic_rebalancing()` - 动态再平衡
//! - `momentum_strategy()` - 动量策略
//! - `mean_reversion()` - 均值回归
//!
//! ### 批量操作指令
//! - `batch_trade_index_token()` - 批量交易指数代币
//! - `batch_process_index_token()` - 批量处理指数代币
//! - `batch_manage_index_token()` - 批量管理指数代币
//! - `batch_sync_index_token()` - 批量同步指数代币
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

// 指数管理指令
pub mod rebalance;
pub mod weight_adjustment;
pub mod constituent_update;
pub mod performance_tracking;

// 交易操作指令
pub mod buy;
pub mod sell;
pub mod swap;
pub mod quote;

// 高级功能指令
pub mod dividend_distribution;
pub mod voting_rights;
pub mod governance;
pub mod fee_management;

// 策略指令
pub mod dynamic_rebalancing;
pub mod momentum_strategy;
pub mod mean_reversion;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出指数管理指令
pub use rebalance::*;
pub use weight_adjustment::*;
pub use constituent_update::*;
pub use performance_tracking::*;

// 重新导出交易操作指令
pub use buy::*;
pub use sell::*;
pub use swap::*;
pub use quote::*;

// 重新导出高级功能指令
pub use dividend_distribution::*;
pub use voting_rights::*;
pub use governance::*;
pub use fee_management::*;

// 重新导出策略指令
pub use dynamic_rebalancing::*;
pub use momentum_strategy::*;
pub use mean_reversion::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use rebalance::{RebalanceIndexParams, RebalanceIndex, RebalanceAdjustmentMechanism, RebalanceTrigger, RebalanceResult};
pub use weight_adjustment::{WeightAdjustmentParams, WeightAdjustment, WeightAdjustmentType, WeightAdjustmentStrategy, WeightAdjustmentResult};
pub use constituent_update::{ConstituentUpdateParams, ConstituentUpdate, ConstituentUpdateType, ConstituentUpdateStrategy, ConstituentInfo, ConstituentUpdateResult};
pub use performance_tracking::{PerformanceTrackingParams, PerformanceTracking, PerformanceTrackingType, PerformanceMetricType, PerformanceMetric, PerformanceTrackingResult};
pub use dividend_distribution::{DividendDistributionParams, DividendDistribution, DividendDistributionType, DividendDistributionStrategy, DividendInfo, DividendDistributionResult};
pub use dynamic_rebalancing::{DynamicRebalancingParams, DynamicRebalancing, DynamicRebalancingTrigger, DynamicRebalancingStrategy, DynamicRebalancingConfig, DynamicRebalancingResult};
pub use batch::{BatchTradeIndexTokenParams, BatchProcessIndexTokenParams, BatchManageIndexTokenParams, BatchSyncIndexTokenParams, BatchIndexToken, BatchOperationType, BatchTradeType, BatchProcessType, BatchManageType, BatchSyncType, BatchOperationResult}; 