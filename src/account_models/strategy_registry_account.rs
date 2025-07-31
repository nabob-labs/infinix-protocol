//!
//! 策略注册账户（PDA 持久化）
//! 该账户用于管理所有已注册的策略元数据，支持策略的注册、切换、查询等操作。
//! 采用 Anchor #[account] 宏声明，PDA 账户持久化，遵循 Solana/Anchor 最佳实践。
//!
//! # 设计说明
//! - 统一管理策略注册表，便于策略的动态扩展、权限控制和生命周期管理。
//! - 采用 PDA（Program Derived Address）机制，确保账户唯一性与安全性。
//! - 支持 Anchor 的 InitSpace、max_len、账户元数据自动管理等最佳实践。
//! - 便于后续集成策略升级、版本控制、审计等功能。

use anchor_lang::prelude::*; // Anchor 预导入，包含账户声明、宏、类型、Context、Result等，确保账户类型声明和生命周期管理符合Anchor最佳实践
use crate::state::common::BaseAccount; // 引入通用账户基础信息结构体BaseAccount，便于权限、生命周期、审计等统一管理
use crate::strategies::StrategyConfig; // 引入策略配置结构体，便于策略注册和查询

/// 策略注册账户结构体
/// - 通过 PDA 持久化存储，管理策略元数据列表
/// - 采用 Anchor #[account] 宏声明，支持 InitSpace 自动空间计算
#[account] // Anchor账户声明宏，自动实现生命周期、权限、序列化等校验逻辑
#[derive(InitSpace)] // Anchor自动推断账户所需空间，便于部署和升级
pub struct StrategyRegistryAccount {
    /// 账户基础信息（通用字段，包含元数据、权限等）
    pub base: BaseAccount, // 通用账户基础信息，便于权限、生命周期、审计等统一管理
    /// 策略注册表（策略 ID 到元数据的映射，最多支持 64 个策略）
    /// - 采用 Anchor #[max_len(64)] 属性，限制最大长度，防止溢出
    #[max_len(64)]
    pub strategies: Vec<StrategyMeta>, // 策略元数据列表，支持动态扩展
}

/// 策略元数据结构体
/// - 记录单个策略的关键信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)] // 支持Anchor序列化、克隆、调试、自动空间推断
pub struct StrategyMeta {
    /// 策略唯一 ID
    pub id: u64, // 策略唯一标识，便于动态查找和切换
    /// 策略配置（包含参数、逻辑等）
    pub config: StrategyConfig, // 策略配置结构体，支持灵活扩展
    /// 策略创建者公钥
    pub creator: Pubkey, // 策略创建者身份标识
    /// 策略创建时间（Unix 时间戳）
    pub created_at: i64, // 策略注册时间，便于审计
    /// 策略最后更新时间（Unix 时间戳）
    pub last_updated: i64, // 策略元数据最后更新时间
    /// 策略是否处于激活状态
    pub is_active: bool, // 策略激活状态，便于动态切换
}

impl StrategyRegistryAccount {
    /// 账户初始化所需空间常量（单位：字节）
    /// - 8 字节 discriminator
    /// - BaseAccount 占用空间
    /// - 4 字节 Vec 长度
    /// - 64 个 StrategyMeta 的最大空间
    pub const INIT_SPACE: usize = 8 + std::mem::size_of::<BaseAccount>() + 4 + (64 * std::mem::size_of::<StrategyMeta>()); // 账户初始化空间常量，便于Anchor自动分配

    /// 注册新策略
    /// - 参数 id: 策略唯一 ID
    /// - 参数 config: 策略配置结构体
    /// - 参数 creator: 策略创建者公钥
    /// - 返回 anchor_lang::Result<()>，成功则策略被添加到注册表
    pub fn register_strategy(&mut self, id: u64, config: StrategyConfig, creator: Pubkey) -> anchor_lang::Result<()> {
        let now = Clock::get()?.unix_timestamp; // 获取当前时间戳，作为策略创建和更新时间
        let meta = StrategyMeta {
            id,                 // 策略唯一 ID
            config,             // 策略配置结构体
            creator,            // 策略创建者公钥
            created_at: now,    // 创建时间
            last_updated: now,  // 最后更新时间
            is_active: true,    // 新注册策略默认激活
        };
        self.strategies.push(meta); // 将新策略添加到注册表
        self.base.touch()?; // 更新账户元数据（如 last_updated 字段）
        Ok(()) // 返回成功
    }

    /// 切换激活策略
    /// - 参数 from: 需停用的策略 ID
    /// - 参数 to: 需激活的策略 ID
    /// - 返回 anchor_lang::Result<()>，成功则切换激活状态
    pub fn switch_strategy(&mut self, from: u64, to: u64) -> anchor_lang::Result<()> {
        let now = Clock::get()?.unix_timestamp; // 获取当前时间戳，作为切换操作的更新时间
        for strat in &mut self.strategies {
            if strat.id == from {
                strat.is_active = false; // 停用原激活策略，并更新时间
                strat.last_updated = now;
            }
            if strat.id == to {
                strat.is_active = true; // 激活目标策略，并更新时间
                strat.last_updated = now;
            }
        }
        self.base.touch()?; // 更新账户元数据
        Ok(())
    }

    /// 查询策略元数据
    /// - 参数 id: 策略唯一 ID
    /// - 返回 Option<&StrategyMeta>，找到则返回引用，否则为 None
    pub fn query_strategy(&self, id: u64) -> Option<&StrategyMeta> {
        self.strategies.iter().find(|s| s.id == id) // 在策略列表中查找 ID 匹配的策略，找到则返回引用，否则返回 None
    }
} 