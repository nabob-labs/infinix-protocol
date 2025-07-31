//!
//! DEX 注册账户（PDA 持久化）
//! 该账户用于管理所有已注册的 DEX 元数据，支持 DEX 的注册、切换、查询等操作。
//! 采用 Anchor #[account] 宏声明，PDA 账户持久化，遵循 Solana/Anchor 最佳实践。
//!
//! # 设计说明
//! - 统一管理 DEX 注册表，便于 DEX 的动态扩展、权限控制和生命周期管理。
//! - 采用 PDA（Program Derived Address）机制，确保账户唯一性与安全性。
//! - 支持 Anchor 的 InitSpace、max_len、账户元数据自动管理等最佳实践。
//! - 便于后续集成 DEX 升级、版本控制、审计等功能。

use anchor_lang::prelude::*; // Anchor 预导入，包含账户声明、宏、类型、Context、Result等，确保账户类型声明和生命周期管理符合Anchor最佳实践
use crate::state::common::BaseAccount; // 引入通用账户基础信息结构体BaseAccount，便于权限、生命周期、审计等统一管理

/// DEX 注册账户结构体
/// - 通过 PDA 持久化存储，管理 DEX 元数据列表
/// - 采用 Anchor #[account] 宏声明，支持 InitSpace 自动空间计算
#[account] // Anchor账户声明宏，自动实现生命周期、权限、序列化等校验逻辑
#[derive(InitSpace)] // Anchor自动推断账户所需空间，便于部署和升级
pub struct DexRegistryAccount {
    /// 账户基础信息（通用字段，包含元数据、权限等）
    pub base: BaseAccount, // 通用账户基础信息，便于权限、生命周期、审计等统一管理
    /// DEX 注册表（DEX 名称到元数据的映射，最多支持 32 个 DEX）
    /// - 采用 Anchor #[max_len(32)] 属性，限制最大长度，防止溢出
    #[max_len(32)]
    pub dexes: Vec<DexMeta>, // DEX 元数据列表，支持动态扩展
}

/// DEX 元数据结构体
/// - 记录单个 DEX 的关键信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)] // 支持Anchor序列化、克隆、调试、自动空间推断
pub struct DexMeta {
    /// DEX 名称（唯一标识）
    #[max_len(32)]
    pub name: String, // DEX 唯一名称，便于动态查找和切换
    /// DEX 创建者公钥
    pub creator: Pubkey, // DEX 创建者身份标识
    /// DEX 创建时间（Unix 时间戳）
    pub created_at: i64, // DEX 注册时间，便于审计
    /// DEX 最后更新时间（Unix 时间戳）
    pub last_updated: i64, // DEX 元数据最后更新时间
    /// DEX 是否处于激活状态
    pub is_active: bool, // DEX 激活状态，便于动态切换
}

impl DexRegistryAccount {
    /// 账户初始化所需空间常量（单位：字节）
    /// - 8 字节 discriminator
    /// - BaseAccount 占用空间
    /// - 4 字节 Vec 长度
    /// - 32 个 DexMeta 的最大空间
    pub const INIT_SPACE: usize = 8 + std::mem::size_of::<BaseAccount>() + 4 + (32 * std::mem::size_of::<DexMeta>()); // 账户初始化空间常量，便于Anchor自动分配

    /// 注册新 DEX
    /// - 参数 name: DEX 名称（唯一）
    /// - 参数 creator: DEX 创建者公钥
    /// - 返回 anchor_lang::Result<()>，成功则 DEX 被添加到注册表
    /// - 若名称重复未做去重，需由上层业务保证唯一性
    /// - Anchor 最佳实践：建议在上层指令中做唯一性校验
    pub fn register_dex(&mut self, name: String, creator: Pubkey) -> anchor_lang::Result<()> {
        let now = Clock::get()?.unix_timestamp; // 获取当前时间戳，作为 DEX 创建和更新时间
        let meta = DexMeta {
            name: name.clone(), // DEX 名称，唯一标识
            creator,           // DEX 创建者公钥
            created_at: now,   // 创建时间
            last_updated: now, // 最后更新时间
            is_active: true,   // 新注册 DEX 默认激活
        };
        self.dexes.push(meta); // 将新 DEX 添加到注册表
        self.base.touch()?; // 更新账户元数据（如 last_updated 字段）
        Ok(()) // 返回成功
    }

    /// 切换激活 DEX
    /// - 参数 from: 需停用的 DEX 名称
    /// - 参数 to: 需激活的 DEX 名称
    /// - 返回 anchor_lang::Result<()>，成功则切换激活状态
    /// - 若名称不存在则无操作，需由上层保证名称有效
    /// - Anchor 最佳实践：建议在上层指令中做存在性校验
    pub fn switch_dex(&mut self, from: &str, to: &str) -> anchor_lang::Result<()> {
        let now = Clock::get()?.unix_timestamp; // 获取当前时间戳，作为切换操作的更新时间
        for dex in &mut self.dexes {
            if dex.name == from {
                dex.is_active = false; // 停用原激活 DEX，并更新时间
                dex.last_updated = now;
            }
            if dex.name == to {
                dex.is_active = true; // 激活目标 DEX，并更新时间
                dex.last_updated = now;
            }
        }
        self.base.touch()?; // 更新账户元数据
        Ok(())
    }

    /// 查询 DEX 元数据
    /// - 参数 name: DEX 名称
    /// - 返回 Option<&DexMeta>，找到则返回引用，否则为 None
    pub fn query_dex(&self, name: &str) -> Option<&DexMeta> {
        self.dexes.iter().find(|d| d.name == name) // 在 DEX 列表中查找名称匹配的 DEX，找到则返回引用，否则返回 None
    }
} 