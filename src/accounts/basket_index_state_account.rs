//!
//! 通用 BasketIndexState 账户结构体定义
//! 该账户结构体统一支持 index_token、basket、asset 三大功能域，提升账户模型复用性。
//! 采用 Anchor #[derive(Accounts)] 宏声明，便于在指令参数中直接引用。
//!
//! # 设计说明
//! - 统一账户模型，简化多业务场景下的账户声明与管理。
//! - 采用 PDA（Program Derived Address）机制，确保账户唯一性与安全性。
//! - 严格遵循 Anchor 账户生命周期、权限、初始化、序列化等最佳实践。
//! - 支持通过指令参数灵活区分 index_token、basket、asset 等业务类型。

use crate::state::baskets::BasketIndexState; // 引入统一的篮子状态结构体BasketIndexState，作为账户数据存储核心，便于资产、篮子、指数等多业务场景的数据复用和一致性管理
use anchor_lang::prelude::*; // Anchor 预导入，包含账户声明、宏、类型、Context、Result等，确保账户类型声明和生命周期管理符合Anchor最佳实践

/// 通用账户结构体，支持 index_token、basket、asset 三大功能域
/// - 通过 PDA seeds 生成唯一账户地址，确保账户安全与可追溯性
/// - 采用 Anchor #[derive(Accounts)] 宏声明，便于指令参数复用
/// - 账户初始化参数兼容多业务场景，支持通过指令参数区分业务类型
#[derive(Accounts)] // Anchor 宏，自动实现账户生命周期、权限、序列化等校验逻辑
pub struct BasketIndexStateAccount<'info> { // 通用账户结构体，生命周期参数 'info 由 Anchor 自动推断
    /// 组合篮子主账户（PDA，持久化存储 BasketIndexState 数据）
    /// - space: 8 + BasketIndexState::INIT_SPACE 字节，满足账户数据扩展需求
    /// - seeds: ["basket_index_state", authority.key().as_ref()]，确保账户唯一性
    #[account(
        init, // Anchor 自动初始化账户
        payer = authority, // authority 账户作为初始化费用支付者
        space = 8 + BasketIndexState::INIT_SPACE, // 分配账户空间，8字节为Anchor头部，INIT_SPACE为业务数据空间
        seeds = [b"basket_index_state", authority.key().as_ref()], // PDA seeds，确保账户唯一性
        bump // Anchor 自动推断bump种子，防止PDA碰撞
    )]
    pub basket_index: Account<'info, BasketIndexState>, // 主账户，持久化存储业务数据，类型安全，Anchor自动校验生命周期
    /// 操作权限签名者
    /// - 必须为 mut，支持账户初始化和后续操作
    #[account(mut)] // Anchor 校验该账户为可变，支持写操作
    pub authority: Signer<'info>, // 账户操作权限签名者，Anchor 自动校验签名和生命周期
    /// 系统程序（用于账户初始化）
    /// - Anchor 自动校验 system_program 的合法性
    pub system_program: Program<'info, System>, // 系统程序，Anchor 自动校验，确保账户初始化安全
}

// 兼容原有功能域的账户初始化参数，可通过指令参数区分业务类型
