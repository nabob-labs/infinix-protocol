//!
//! 预言机注册账户（PDA 持久化）
//! 该账户用于管理所有已注册的预言机元数据，支持预言机的注册、切换、查询等操作。
//! 采用 Anchor #[account] 宏声明，PDA 账户持久化，遵循 Solana/Anchor 最佳实践。
//!
//! # 设计说明
//! - 统一管理预言机注册表，便于预言机的动态扩展、权限控制和生命周期管理。
//! - 采用 PDA（Program Derived Address）机制，确保账户唯一性与安全性。
//! - 支持 Anchor 的 InitSpace、max_len、账户元数据自动管理等最佳实践。
//! - 便于后续集成预言机升级、版本控制、审计等功能。

use anchor_lang::prelude::*; // Anchor 预导入，包含账户声明、宏、类型、Context、Result等，确保账户类型声明和生命周期管理符合Anchor最佳实践
use crate::state::common::BaseAccount; // 引入通用账户基础信息结构体BaseAccount，便于权限、生命周期、审计等统一管理

/// 预言机注册账户结构体
/// - 通过 PDA 持久化存储，管理预言机元数据列表
/// - 采用 Anchor #[account] 宏声明，支持 InitSpace 自动空间计算
#[account] // Anchor账户声明宏，自动实现生命周期、权限、序列化等校验逻辑
#[derive(InitSpace)] // Anchor自动推断账户所需空间，便于部署和升级
pub struct OracleRegistryAccount {
    /// 账户基础信息（通用字段，包含元数据、权限等）
    pub base: BaseAccount, // 通用账户基础信息，便于权限、生命周期、审计等统一管理
    /// 预言机注册表（名称到元数据的映射，最多支持 32 个预言机）
    /// - 采用 Anchor #[max_len(32)] 属性，限制最大长度，防止溢出
    #[max_len(32)]
    pub oracles: Vec<OracleMeta>, // 预言机元数据列表，支持动态扩展
}

/// 预言机元数据结构体
/// - 记录单个预言机的关键信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)] // 支持Anchor序列化、克隆、调试、自动空间推断
pub struct OracleMeta {
    /// 预言机名称（唯一标识）
    #[max_len(32)]
    pub name: String, // 预言机唯一名称，便于动态查找和切换
    /// 预言机创建者公钥
    pub creator: Pubkey, // 预言机创建者身份标识
    /// 预言机创建时间（Unix 时间戳）
    pub created_at: i64, // 预言机注册时间，便于审计
    /// 预言机最后更新时间（Unix 时间戳）
    pub last_updated: i64, // 预言机元数据最后更新时间
    /// 预言机是否处于激活状态
    pub is_active: bool, // 预言机激活状态，便于动态切换
}

impl OracleRegistryAccount {
    /// 账户初始化所需空间常量（单位：字节）
    /// - 8 字节 discriminator
    /// - BaseAccount 占用空间
    /// - 4 字节 Vec 长度
    /// - 32 个 OracleMeta 的最大空间
    pub const INIT_SPACE: usize = 8 + std::mem::size_of::<BaseAccount>() + 4 + (32 * std::mem::size_of::<OracleMeta>()); // 账户初始化空间常量，便于Anchor自动分配

    /// 注册新预言机
    /// - 参数 name: 预言机名称（唯一）
    /// - 参数 creator: 预言机创建者公钥
    /// - 返回 anchor_lang::Result<()>，成功则预言机被添加到注册表
    /// - 若名称重复未做去重，需由上层业务保证唯一性
    /// - Anchor 最佳实践：建议在上层指令中做唯一性校验
    pub fn register_oracle(&mut self, name: String, creator: Pubkey) -> anchor_lang::Result<()> {
        let now = Clock::get()?.unix_timestamp; // 获取当前时间戳，作为预言机创建和更新时间
        let meta = OracleMeta {
            name: name.clone(), // 预言机名称，唯一标识
            creator,           // 预言机创建者公钥
            created_at: now,   // 创建时间
            last_updated: now, // 最后更新时间
            is_active: true,   // 新注册预言机默认激活
        };
        self.oracles.push(meta); // 将新预言机添加到注册表
        self.base.touch()?; // 更新账户元数据（如 last_updated 字段）
        Ok(()) // 返回成功
    }

    /// 切换激活预言机
    /// - 参数 from: 需停用的预言机名称
    /// - 参数 to: 需激活的预言机名称
    /// - 返回 anchor_lang::Result<()>，成功则切换激活状态
    /// - 若名称不存在则无操作，需由上层保证名称有效
    /// - Anchor 最佳实践：建议在上层指令中做存在性校验
    pub fn switch_oracle(&mut self, from: &str, to: &str) -> anchor_lang::Result<()> {
        let now = Clock::get()?.unix_timestamp; // 获取当前时间戳，作为切换操作的更新时间
        for oracle in &mut self.oracles {
            if oracle.name == from {
                oracle.is_active = false; // 停用原激活预言机，并更新时间
                oracle.last_updated = now;
            }
            if oracle.name == to {
                oracle.is_active = true; // 激活目标预言机，并更新时间
                oracle.last_updated = now;
            }
        }
        self.base.touch()?; // 更新账户元数据
        Ok(())
    }

    /// 查询预言机元数据
    /// - 参数 name: 预言机名称
    /// - 返回 Option<&OracleMeta>，找到则返回引用，否则为 None
    pub fn query_oracle(&self, name: &str) -> Option<&OracleMeta> {
        self.oracles.iter().find(|o| o.name == name) // 在预言机列表中查找名称匹配的预言机，找到则返回引用，否则返回 None
    }
} 