//!
//! 账户模块统一入口
//! 本文件统一 re-export 各账户相关子模块，并定义部分通用账户参数结构体。
//!
//! # 设计说明
//! - 通过 mod.rs 聚合导出所有账户相关子模块，便于统一管理和外部调用。
//! - 支持 Anchor #[derive(Accounts)] 宏声明的通用账户参数结构体，便于指令参数扩展和复用。
//! - 可根据实际业务需求灵活扩展账户参数结构体。

pub mod asset; // 资产账户子模块，统一管理资产相关账户类型，便于资产相关指令复用
pub mod basket; // 组合篮子账户子模块，统一管理篮子相关账户类型，便于篮子相关指令复用
pub mod basket_index_state_account; // 通用篮子/资产/指数账户结构体子模块，统一账户模型
pub mod index_token; // 指数代币账户子模块，统一管理指数相关账户类型，便于指数相关指令复用
pub use basket_index_state_account::*; // 统一 re-export 通用账户结构体，便于外部直接引用和复用

/// 创建组合篮子账户参数结构体（示例）
/// - 可根据实际业务需求扩展字段
/// - Anchor #[derive(Accounts)] 宏声明，便于指令参数校验和生命周期管理
#[derive(Accounts)] // Anchor 宏，自动实现账户生命周期、权限、序列化等校验逻辑
pub struct CreateBasket {} // 示例结构体，实际可根据业务需求扩展字段，当前为空

/// 执行交易账户参数结构体（示例）
/// - 适用于各类资产、篮子、指数等交易指令
#[derive(Accounts)] // Anchor 宏，自动实现账户生命周期、权限、序列化等校验逻辑
pub struct ExecuteTrade {} // 示例结构体，实际可根据业务需求扩展字段，当前为空

/// 风险检查账户参数结构体（示例）
/// - 适用于风控、合规等业务场景
#[derive(Accounts)] // Anchor 宏，自动实现账户生命周期、权限、序列化等校验逻辑
pub struct CheckRisk {} // 示例结构体，实际可根据业务需求扩展字段，当前为空

/// 流动性聚合账户参数结构体（示例）
/// - 适用于多DEX/AMM流动性聚合场景
#[derive(Accounts)] // Anchor 宏，自动实现账户生命周期、权限、序列化等校验逻辑
pub struct AggregateLiquidity {} // 示例结构体，实际可根据业务需求扩展字段，当前为空
