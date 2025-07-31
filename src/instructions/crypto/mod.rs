//! 加密货币 (Crypto) 资产类型指令集模块
//! 
//! 本模块实现完整的加密货币交易功能，支持跨链桥接、闪电贷、MEV保护等高级功能。
//! 严格遵循最小功能单元原则，每个指令函数职责单一，便于维护和扩展。
//! 
//! ## 功能指令集
//! 
//! ### 基础操作指令
//! - `mint_crypto()` - 增发加密货币
//! - `burn_crypto()` - 销毁加密货币  
//! - `transfer_crypto()` - 转账加密货币
//! - `query_crypto()` - 查询加密货币余额
//! 
//! ### 交易操作指令
//! - `buy_crypto()` - 买入加密货币
//! - `sell_crypto()` - 卖出加密货币
//! - `swap_crypto()` - 兑换加密货币
//! - `quote_crypto()` - 获取加密货币报价
//! 
//! ### 高级操作指令
//! - `combine_crypto()` - 合并加密货币
//! - `batch_combine_crypto()` - 批量合并加密货币
//! - `algo_combine_crypto()` - 算法合并加密货币
//! - `strategy_combine_crypto()` - 策略合并加密货币
//! - `split_crypto()` - 拆分加密货币
//! - `batch_split_crypto()` - 批量拆分加密货币
//! - `algo_split_crypto()` - 算法拆分加密货币
//! - `strategy_split_crypto()` - 策略拆分加密货币
//! - `freeze_crypto()` - 冻结加密货币
//! - `emergency_freeze_crypto()` - 紧急冻结加密货币
//! - `batch_freeze_crypto()` - 批量冻结加密货币
//! - `conditional_freeze_crypto()` - 条件冻结加密货币
//! - `unfreeze_crypto()` - 解冻加密货币
//! - `batch_unfreeze_crypto()` - 批量解冻加密货币
//! - `conditional_unfreeze_crypto()` - 条件解冻加密货币
//! - `auto_unfreeze_crypto()` - 自动解冻加密货币
//! - `authorize_crypto()` - 授权加密货币操作
//! - `batch_authorize_crypto()` - 批量授权加密货币
//! - `role_authorize_crypto()` - 角色授权加密货币
//! - `temporary_authorize_crypto()` - 临时授权加密货币
//! 
//! ### 批量操作指令
//! - `batch_trade_crypto()` - 批量交易加密货币
//! - `batch_process_crypto()` - 批量处理加密货币
//! - `batch_manage_crypto()` - 批量管理加密货币
//! - `batch_sync_crypto()` - 批量同步加密货币
//! 
//! ### 策略操作指令
//! - `strategy_trade_crypto()` - 策略交易加密货币
//! - `batch_strategy_trade_crypto()` - 批量策略交易加密货币
//! - `algo_strategy_trade_crypto()` - 算法策略交易加密货币
//! - `risk_controlled_strategy_trade_crypto()` - 风险控制策略交易加密货币
//! - `algo_trade_crypto()` - 算法交易加密货币
//! - `batch_algo_trade_crypto()` - 批量算法交易加密货币
//! - `twap_algo_trade_crypto()` - TWAP算法交易加密货币
//! - `vwap_algo_trade_crypto()` - VWAP算法交易加密货币
//! - `smart_routing_algo_trade_crypto()` - 智能路由算法交易加密货币
//! - `arbitrage_crypto()` - 套利交易加密货币
//! - `batch_arbitrage_crypto()` - 批量套利交易加密货币
//! - `multi_dex_arbitrage_crypto()` - 多DEX套利交易加密货币
//! - `cross_market_arbitrage_crypto()` - 跨市场套利交易加密货币
//! - `statistical_arbitrage_crypto()` - 统计套利交易加密货币
//! - `detect_arbitrage_opportunities_crypto()` - 套利机会检测加密货币
//! 
//! ### 高级功能指令
//! - `cross_chain_bridge_crypto()` - 跨链桥接
//! - `verify_cross_chain_bridge_crypto()` - 跨链桥接验证
//! - `flash_loan_crypto()` - 闪电贷
//! - `flash_loan_arbitrage_crypto()` - 闪电贷套利
//! - `flash_loan_repay_crypto()` - 闪电贷还款
//! - `mev_protection_crypto()` - MEV保护
//! - `mev_detection_crypto()` - MEV检测
//! - `anti_arbitrage_protection_crypto()` - 反套利保护
//! - `slippage_protection_crypto()` - 滑点保护
//! - `slippage_detection_crypto()` - 滑点检测
//! - `dynamic_slippage_adjustment_crypto()` - 动态滑点调整
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

// === 高级功能指令 ===
pub mod cross_chain_bridge;  // 跨链桥接指令
pub mod flash_loan;          // 闪电贷指令
pub mod mev_protection;      // MEV保护指令
pub mod slippage_protection; // 滑点保护指令

// === 策略操作指令 ===
pub mod strategy_trade;      // 策略交易指令
pub mod algo_trade;          // 算法交易指令
pub mod arbitrage;           // 套利交易指令

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
pub use cross_chain_bridge::*;
pub use flash_loan::*;
pub use mev_protection::*;
pub use slippage_protection::*;
pub use strategy_trade::*;
pub use algo_trade::*;
pub use arbitrage::*;

// === 重新导出所有结构体和类型 ===
pub use split::{SplitOrder, SplitResult};
pub use freeze::{FreezeOrder, FreezeCondition, FreezeType, FreezeResult};
pub use unfreeze::{UnfreezeOrder, UnfreezeCondition, UnfreezeType, UnfreezeResult};
pub use authorize::{AuthorizeOrder, RoleAuthorization, TemporaryAuthorization, AuthorizeType, AuthorizeResult};
pub use batch::{BatchProcessOrder, BatchManagementOrder, BatchSyncOrder, BatchProcessType, BatchManagementType, BatchSyncType, BatchResult};
pub use cross_chain_bridge::{CrossChainParams, BridgeVerification, BridgeType, BridgeResult};
pub use flash_loan::{FlashLoanParams, FlashLoanArbitrage, FlashLoanRepayment, FlashLoanType, FlashLoanResult};
pub use mev_protection::{MevProtectionParams, MevDetection, AntiArbitrageProtection, MevProtectionType, MevProtectionResult};
pub use slippage_protection::{SlippageProtectionParams, SlippageDetection, DynamicSlippageAdjustment, SlippageProtectionType, SlippageProtectionResult};
pub use strategy_trade::{StrategyTradeCryptoParams};
pub use algo_trade::{AlgoTradeCryptoParams};
pub use arbitrage::{ArbitrageCryptoParams}; 