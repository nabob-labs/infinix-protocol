//! Yield Token (收益代币) 资产类型指令模块
//! 
//! 本模块提供Yield Token资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - 收益功能：生成收益、分配收益、复投收益、收获收益
//! - 高级功能：收益农场、自动复投、收益优化、风险调整
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给YieldTokenService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// 收益功能指令
pub mod generate_yield;
pub mod distribute_yield;
pub mod compound_yield;
pub mod harvest_yield;

// 高级功能指令
pub mod yield_farming;
pub mod auto_compound;
pub mod yield_optimization;
pub mod risk_adjustment;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出收益功能指令
pub use generate_yield::*;
pub use distribute_yield::*;
pub use compound_yield::*;
pub use harvest_yield::*;

// 重新导出高级功能指令
pub use yield_farming::*;
pub use auto_compound::*;
pub use yield_optimization::*;
pub use risk_adjustment::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintYieldParams, MintYield};
pub use burn::{BurnYieldParams, BurnYield};
pub use transfer::{TransferYieldParams, TransferYield};
pub use query::{QueryYieldParams, QueryYield};
pub use generate_yield::{GenerateYieldParams, GenerateYield};
pub use distribute_yield::{DistributeYieldParams, DistributeYield};
pub use compound_yield::{CompoundYieldParams, CompoundYield};
pub use harvest_yield::{HarvestYieldParams, HarvestYield};
pub use yield_farming::{YieldFarmingParams, YieldFarming};
pub use auto_compound::{AutoCompoundParams, AutoCompound};
pub use yield_optimization::{YieldOptimizationParams, YieldOptimization};
pub use risk_adjustment::{RiskAdjustmentParams, RiskAdjustment};
pub use batch::{BatchTradeYieldParams, BatchProcessYieldParams, BatchManageYieldParams, BatchSyncYieldParams, BatchYield}; 