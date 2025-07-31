//! LP Token (Liquidity Provider Token) 资产类型指令模块
//! 
//! 本模块提供LP Token资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - LP特有功能：添加流动性、移除流动性、收获奖励、复投奖励
//! - 交易操作：兑换、报价、无常损失计算、年化收益率计算
//! - 高级功能：流动性挖矿、自动复投、收益优化、风险调整
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给LpTokenService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// LP特有功能指令
pub mod add_liquidity;
pub mod remove_liquidity;
pub mod harvest_rewards;
pub mod compound_rewards;

// 交易操作指令
pub mod swap;
pub mod quote;
pub mod impermanent_loss;
pub mod apy_calculation;

// 高级功能指令
pub mod liquidity_mining;
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

// 重新导出LP特有功能指令
pub use add_liquidity::*;
pub use remove_liquidity::*;
pub use harvest_rewards::*;
pub use compound_rewards::*;

// 重新导出交易操作指令
pub use swap::*;
pub use quote::*;
pub use impermanent_loss::*;
pub use apy_calculation::*;

// 重新导出高级功能指令
pub use liquidity_mining::*;
pub use auto_compound::*;
pub use yield_optimization::*;
pub use risk_adjustment::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintLpTokenParams, MintLpToken};
pub use burn::{BurnLpTokenParams, BurnLpToken};
pub use transfer::{TransferLpTokenParams, TransferLpToken};
pub use query::{QueryLpTokenParams, QueryLpToken};
pub use add_liquidity::{AddLiquidityParams, AddLiquidity};
pub use remove_liquidity::{RemoveLiquidityParams, RemoveLiquidity};
pub use harvest_rewards::{HarvestRewardsParams, HarvestRewards};
pub use compound_rewards::{CompoundRewardsParams, CompoundRewards};
pub use swap::{SwapLpTokenParams, SwapLpToken};
pub use quote::{QuoteLpTokenParams, QuoteLpToken};
pub use impermanent_loss::{ImpermanentLossParams, ImpermanentLoss};
pub use apy_calculation::{ApyCalculationParams, ApyCalculation};
pub use liquidity_mining::{LiquidityMiningParams, LiquidityMining};
pub use auto_compound::{AutoCompoundParams, AutoCompound};
pub use yield_optimization::{YieldOptimizationParams, YieldOptimization};
pub use risk_adjustment::{RiskAdjustmentParams, RiskAdjustment};
pub use batch::{BatchTradeLpTokenParams, BatchProcessLpTokenParams, BatchManageLpTokenParams, BatchSyncLpTokenParams, BatchLpToken}; 