//! Governance Token (治理代币) 资产类型指令模块
//! 
//! 本模块提供Governance Token资产的完整功能指令集，包括：
//! - 基础操作：铸造、销毁、转账、查询
//! - 治理功能：创建提案、投票、执行提案、委托投票
//! - 高级功能：快照投票、二次投票、信念投票、时间锁
//! - 批量操作：批量交易、批量处理、批量管理、批量同步
//! 
//! 设计特点：
//! - 最小功能单元：每个指令功能单一，职责明确
//! - 细粒度设计：支持灵活组合和扩展
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证和边界检查
//! - 权限控制：细粒度的权限验证和管理
//! - 服务层抽象：核心业务逻辑委托给GovernanceTokenService
//! - 事件驱动：完整的事件发射和审计追踪
//! - 错误处理：全面的错误类型和处理机制

// 基础操作指令
pub mod mint;
pub mod burn;
pub mod transfer;
pub mod query;

// 治理功能指令
pub mod create_proposal;
pub mod vote_proposal;
pub mod execute_proposal;
pub mod delegate_votes;

// 高级功能指令
pub mod snapshot_voting;
pub mod quadratic_voting;
pub mod conviction_voting;
pub mod time_lock;

// 批量操作指令
pub mod batch;

// 重新导出基础操作指令
pub use mint::*;
pub use burn::*;
pub use transfer::*;
pub use query::*;

// 重新导出治理功能指令
pub use create_proposal::*;
pub use vote_proposal::*;
pub use execute_proposal::*;
pub use delegate_votes::*;

// 重新导出高级功能指令
pub use snapshot_voting::*;
pub use quadratic_voting::*;
pub use conviction_voting::*;
pub use time_lock::*;

// 重新导出批量操作指令
pub use batch::*;

// 重新导出参数结构体
pub use mint::{MintGovernanceParams, MintGovernance};
pub use burn::{BurnGovernanceParams, BurnGovernance};
pub use transfer::{TransferGovernanceParams, TransferGovernance};
pub use query::{QueryGovernanceParams, QueryGovernance};
pub use create_proposal::{CreateProposalParams, CreateProposal};
pub use vote_proposal::{VoteProposalParams, VoteProposal};
pub use execute_proposal::{ExecuteProposalParams, ExecuteProposal};
pub use delegate_votes::{DelegateVotesParams, DelegateVotes};
pub use snapshot_voting::{SnapshotVotingParams, SnapshotVoting};
pub use quadratic_voting::{QuadraticVotingParams, QuadraticVoting};
pub use conviction_voting::{ConvictionVotingParams, ConvictionVoting};
pub use time_lock::{TimeLockParams, TimeLock};
pub use batch::{BatchTradeGovernanceParams, BatchProcessGovernanceParams, BatchManageGovernanceParams, BatchSyncGovernanceParams, BatchGovernance}; 