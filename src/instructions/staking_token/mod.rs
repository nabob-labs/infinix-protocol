//! Staking Token (质押代币) 资产类型指令模块
//! 
//! 本模块提供Staking Token资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - 质押功能：质押、解质押、领取奖励、复投奖励
//! - 高级功能：锁定质押、提前解质押、惩罚机制、验证者选择
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给StakingTokenService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// 质押功能指令
pub mod stake_tokens;
pub mod unstake_tokens;
pub mod claim_rewards;
pub mod compound_rewards;

// 高级功能指令
pub mod lock_staking;
pub mod early_unstake;
pub mod slashing;
pub mod validator_selection;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出质押功能指令
pub use stake_tokens::*;
pub use unstake_tokens::*;
pub use claim_rewards::*;
pub use compound_rewards::*;

// 重新导出高级功能指令
pub use lock_staking::*;
pub use early_unstake::*;
pub use slashing::*;
pub use validator_selection::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintStakingParams, MintStaking};
pub use burn::{BurnStakingParams, BurnStaking};
pub use transfer::{TransferStakingParams, TransferStaking};
pub use query::{QueryStakingParams, QueryStaking};
pub use stake_tokens::{StakeTokensParams, StakeTokens};
pub use unstake_tokens::{UnstakeTokensParams, UnstakeTokens};
pub use claim_rewards::{ClaimRewardsParams, ClaimRewards};
pub use compound_rewards::{CompoundRewardsParams, CompoundRewards};
pub use lock_staking::{LockStakingParams, LockStaking};
pub use early_unstake::{EarlyUnstakeParams, EarlyUnstake};
pub use slashing::{SlashingParams, Slashing};
pub use validator_selection::{ValidatorSelectionParams, ValidatorSelection};
pub use batch::{BatchTradeStakingParams, BatchProcessStakingParams, BatchManageStakingParams, BatchSyncStakingParams, BatchStaking}; 