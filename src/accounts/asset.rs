//!
//! 资产账户相关类型定义
//! 本文件已将资产相关账户结构体统一合并至 basket_index_state_account.rs::BasketIndexStateAccount。
//! 请在指令实现中统一使用该账户结构体，提升代码复用性和可维护性。
//!
//! # 设计说明
//! - 资产账户采用统一的 `BasketIndexStateAccount` 结构体，便于资产、篮子、指数等多业务场景复用。
//! - 通过类型别名简化 Anchor 指令参数类型声明，提升可读性和可维护性。
//! - 严格遵循 Anchor 账户声明规范，便于后续权限校验、序列化和跨程序调用。

use crate::state::baskets::BasketIndexState; // 引入统一的篮子状态结构体BasketIndexState，作为资产账户的底层数据结构，便于资产、篮子、指数等多业务场景的数据复用和一致性管理
use anchor_lang::prelude::*; // Anchor 预导入，包含账户声明、宏、类型、Context、Result等，确保账户类型声明和生命周期管理符合Anchor最佳实践

/// 资产账户创建类型别名
/// - 统一使用 `BasketIndexStateAccount` 作为资产账户的 Anchor 账户参数类型，便于所有资产相关指令统一账户声明和校验
/// - 便于指令中直接引用，提升复用性和一致性，减少重复代码
/// - 支持 Anchor 的账户校验、生命周期管理、权限控制等最佳实践，确保合约安全性和可维护性
pub type CreateAsset<'info> = BasketIndexStateAccount<'info>; // 类型别名声明，将CreateAsset<'info>映射为BasketIndexStateAccount<'info>，生命周期参数 'info 由 Anchor 自动推断和管理，确保账户生命周期安全
