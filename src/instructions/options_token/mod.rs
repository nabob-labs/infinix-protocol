//! Options Token (期权代币) 资产类型指令模块
//! 
//! 本模块提供Options Token资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - 期权功能：创建期权、行权、到期、对冲
//! - 高级功能：期权定价、希腊字母计算、波动率曲面、风险指标
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给OptionsTokenService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// 期权功能指令
pub mod create_option;
pub mod exercise_option;
pub mod expire_option;
pub mod hedge_option;

// 高级功能指令
pub mod options_pricing;
pub mod greeks_calculation;
pub mod volatility_surface;
pub mod risk_metrics;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出期权功能指令
pub use create_option::*;
pub use exercise_option::*;
pub use expire_option::*;
pub use hedge_option::*;

// 重新导出高级功能指令
pub use options_pricing::*;
pub use greeks_calculation::*;
pub use volatility_surface::*;
pub use risk_metrics::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintOptionsParams, MintOptions};
pub use burn::{BurnOptionsParams, BurnOptions};
pub use transfer::{TransferOptionsParams, TransferOptions};
pub use query::{QueryOptionsParams, QueryOptions};
pub use create_option::{CreateOptionParams, CreateOption};
pub use exercise_option::{ExerciseOptionParams, ExerciseOption};
pub use expire_option::{ExpireOptionParams, ExpireOption};
pub use hedge_option::{HedgeOptionParams, HedgeOption};
pub use options_pricing::{OptionsPricingParams, OptionsPricing};
pub use greeks_calculation::{GreeksCalculationParams, GreeksCalculation};
pub use volatility_surface::{VolatilitySurfaceParams, VolatilitySurface};
pub use risk_metrics::{RiskMetricsParams, RiskMetrics};
pub use batch::{BatchTradeOptionsParams, BatchProcessOptionsParams, BatchManageOptionsParams, BatchSyncOptionsParams, BatchOptions};