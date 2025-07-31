//! 稳定币 (Stablecoin) 资产类型指令集模块
//! 
//! 本模块实现完整的稳定币交易功能，支持稳定机制、风险控制、治理投票等特有功能。
//! 严格遵循最小功能单元原则，每个指令函数职责单一，便于维护和扩展。
//! 
//! ## 功能指令集
//! 
//! ### 基础操作指令
//! - `mint_stablecoin()` - 增发稳定币
//! - `burn_stablecoin()` - 销毁稳定币  
//! - `transfer_stablecoin()` - 转账稳定币
//! - `query_stablecoin()` - 查询稳定币余额
//! 
//! ### 交易操作指令
//! - `buy_stablecoin()` - 买入稳定币
//! - `sell_stablecoin()` - 卖出稳定币
//! - `swap_stablecoin()` - 兑换稳定币
//! - `quote_stablecoin()` - 获取稳定币报价
//! 
//! ### 高级操作指令
//! - `combine_stablecoin()` - 合并稳定币
//! - `split_stablecoin()` - 拆分稳定币
//! - `freeze_stablecoin()` - 冻结稳定币
//! - `unfreeze_stablecoin()` - 解冻稳定币
//! - `authorize_stablecoin()` - 授权稳定币操作
//! 
//! ### 批量操作指令
//! - `batch_trade_stablecoin()` - 批量交易稳定币
//! - `batch_process_stablecoin()` - 批量处理稳定币
//! - `batch_manage_stablecoin()` - 批量管理稳定币
//! - `batch_sync_stablecoin()` - 批量同步稳定币
//! 
//! ### 稳定机制指令
//! - `rebalance_stablecoin()` - 稳定币再平衡
//! - `peg_maintenance_stablecoin()` - 锚定维护稳定币
//! - `soft_peg_maintenance_stablecoin()` - 软锚定维护稳定币
//! - `hard_peg_maintenance_stablecoin()` - 硬锚定维护稳定币
//! - `algorithmic_peg_maintenance_stablecoin()` - 算法锚定维护稳定币
//! - `collateral_ratio_stablecoin()` - 抵押率管理稳定币
//! - `add_collateral_stablecoin()` - 添加抵押品稳定币
//! - `remove_collateral_stablecoin()` - 移除抵押品稳定币
//! - `trigger_liquidation_stablecoin()` - 触发清算稳定币
//! - `liquidation_stablecoin()` - 清算机制稳定币
//! - `automatic_liquidation_stablecoin()` - 自动清算稳定币
//! - `manual_liquidation_stablecoin()` - 手动清算稳定币
//! - `emergency_liquidation_stablecoin()` - 紧急清算稳定币
//! 
//! ### 风险管理指令
//! - `risk_assessment_stablecoin()` - 风险评估稳定币
//! - `collateral_risk_assessment_stablecoin()` - 抵押品风险评估稳定币
//! - `market_risk_assessment_stablecoin()` - 市场风险评估稳定币
//! - `liquidity_risk_assessment_stablecoin()` - 流动性风险评估稳定币
//! - `stress_test_stablecoin()` - 压力测试稳定币
//! - `market_crash_stress_test_stablecoin()` - 市场崩盘压力测试稳定币
//! - `liquidity_crisis_stress_test_stablecoin()` - 流动性危机压力测试稳定币
//! - `systemic_risk_stress_test_stablecoin()` - 系统性风险压力测试稳定币
//! - `emergency_pause_stablecoin()` - 紧急暂停稳定币
//! - `emergency_resume_stablecoin()` - 紧急恢复稳定币
//! - `market_anomaly_emergency_pause_stablecoin()` - 市场异常紧急暂停稳定币
//! - `security_breach_emergency_pause_stablecoin()` - 安全漏洞紧急暂停稳定币
//! - `circuit_breaker_stablecoin()` - 熔断机制稳定币
//! - `price_circuit_breaker_stablecoin()` - 价格熔断稳定币
//! - `trading_circuit_breaker_stablecoin()` - 交易熔断稳定币
//! - `volatility_circuit_breaker_stablecoin()` - 波动率熔断稳定币
//! 
//! ### 治理指令
//! - `governance_vote_stablecoin()` - 治理投票稳定币
//! - `delegate_vote_stablecoin()` - 委托投票稳定币
//! - `snapshot_vote_stablecoin()` - 快照投票稳定币
//! - `quadratic_vote_stablecoin()` - 二次投票稳定币
//! - `proposal_submit_stablecoin()` - 提案提交稳定币
//! - `parameter_update_stablecoin()` - 参数更新稳定币
//! 
//! ## 设计原则
//! - 最小功能单元：每个函数只做一件事，职责单一
//! - 类型安全：严格的类型检查和边界校验
//! - 可插拔设计：支持运行时动态扩展和替换
//! - 性能优化：高效的批量操作和算法执行
//! - 合规性：符合金融监管和审计要求

// === 基础操作指令 ===
pub mod mint;           // 增发指令
pub mod burn;           // 销毁指令
pub mod transfer;       // 转账指令
pub mod query;          // 查询指令

// === 交易操作指令 ===
pub mod buy;            // 买入指令
pub mod sell;           // 卖出指令
pub mod swap;           // 兑换指令
pub mod quote;          // 报价指令

// === 高级操作指令 ===
pub mod combine;        // 合并指令
pub mod split;          // 拆分指令
pub mod freeze;         // 冻结指令
pub mod unfreeze;       // 解冻指令
pub mod authorize;      // 授权指令

// === 批量操作指令 ===
pub mod batch;          // 批量操作指令

// === 稳定机制指令 ===
pub mod rebalance;      // 再平衡指令
pub mod peg_maintenance; // 锚定维护指令
pub mod collateral_ratio; // 抵押率管理指令
pub mod liquidation;    // 清算机制指令

// === 风险管理指令 ===
pub mod risk_assessment; // 风险评估指令
pub mod stress_test;     // 压力测试指令
pub mod emergency_pause; // 紧急暂停指令
pub mod circuit_breaker; // 熔断机制指令

// === 治理指令 ===
pub mod governance_vote; // 治理投票指令
pub mod proposal_submit; // 提案提交指令
pub mod parameter_update; // 参数更新指令

// === 重新导出所有指令函数 ===
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;
pub use buy::*;
pub use sell::*;
pub use swap::*;
pub use quote::*;
pub use combine::*;
pub use split::*;
pub use freeze::*;
pub use unfreeze::*;
pub use authorize::*;
pub use batch::*;
pub use rebalance::*;
pub use peg_maintenance::*;
pub use collateral_ratio::*;
pub use liquidation::*;
pub use risk_assessment::*;
pub use stress_test::*;
pub use emergency_pause::*;
pub use circuit_breaker::*;
pub use governance_vote::*;
pub use proposal_submit::*;
pub use parameter_update::*;

// === 重新导出所有结构体和类型 ===
pub use split::{SplitOrder, SplitResult};
pub use freeze::{FreezeOrder, FreezeCondition, FreezeType, FreezeResult};
pub use unfreeze::{UnfreezeOrder, UnfreezeCondition, UnfreezeType, UnfreezeResult};
pub use authorize::{AuthorizeOrder, RoleAuthorization, TemporaryAuthorization, AuthorizeType, AuthorizeResult};
pub use batch::{BatchProcessOrder, BatchManagementOrder, BatchSyncOrder, BatchProcessType, BatchManagementType, BatchSyncType, BatchResult};
pub use rebalance::{RebalanceOrder, RebalanceType, RebalanceResult};
pub use peg_maintenance::{PegMaintenanceOrder, PegType, PegAdjustmentMechanism, PegMaintenanceResult, AdjustmentDirection};
pub use collateral_ratio::{CollateralRatioOrder, CollateralAsset, CollateralRiskLevel, CollateralRatioResult, CollateralRiskStatus};
pub use liquidation::{LiquidationOrder, LiquidationType, LiquidationResult, LiquidationStatus};
pub use risk_assessment::{RiskAssessmentOrder, RiskType, RiskThresholds, StressScenario, RiskAssessmentResult, RiskLevel, RiskMetrics};
pub use stress_test::{StressTestOrder, StressScenario, StressLevels, RiskFactor, RiskFactorType, StressTestResult, StressRiskLevel, StressMetrics};
pub use emergency_pause::{EmergencyPauseOrder, EmergencyType, ProtectionLevel, EmergencyPauseResult, PauseStatus};
pub use circuit_breaker::{CircuitBreakerOrder, CircuitBreakerType, BreakerLevel, CircuitBreakerResult, BreakerStatus};
pub use governance_vote::{GovernanceVoteOrder, VoteType, VoteChoice, GovernanceVoteResult, VoteStatus};
pub use proposal_submit::{ProposalSubmitOrder, ProposalType, ProposalResult};
pub use parameter_update::{ParameterUpdateOrder, ParameterType, ParameterResult}; 