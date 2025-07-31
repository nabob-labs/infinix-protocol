// ========================= 通用状态类型与工具实现 =========================
// 本模块为所有账户基础字段、NAV 计算、费用管理、再平衡、操作统计等提供通用结构体和工具，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、错误、测试等均有详细注释。

#![allow(clippy::too_many_arguments)] // 允许函数参数过多的clippy警告，便于复杂业务接口设计

use anchor_lang::prelude::ProgramError; // 引入自定义错误类型，便于统一错误处理
use crate::version::{ProgramVersion, Versioned, CURRENT_VERSION}; // 引入版本管理相关类型和常量
use anchor_lang::prelude::*; // Anchor 预导入，包含账户、宏、类型、Context、Result等

/// 价格预言机结构体
/// - 记录单一资产的mint和价格
/// - 用于NAV计算、资产估值等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, InitSpace)] // 支持Anchor序列化、克隆、调试、自动空间推断
pub struct PriceFeed {
    /// 资产mint地址
    pub mint: Pubkey, // 资产唯一标识
    /// 当前价格（最小单位）
    pub price: u64, // 资产当前价格，单位为最小分
}

impl PriceFeed {
    /// 校验价格预言机有效性
    /// - 检查mint地址是否为默认值
    /// - 返回Err表示预言机不可用
    pub fn validate(&self) -> anchor_lang::Result<()> {
        if self.mint == Pubkey::default() { // 检查mint是否为默认空值
            return Err(ProgramError::PriceFeedUnavailable.into()); // 返回预言机不可用错误
        }
        Ok(()) // 校验通过
    }
}

/// 通用账户基础字段
/// - 适用于所有Anchor账户，支持版本、权限、激活/暂停、时间戳、PDA bump等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, InitSpace)] // 支持Anchor序列化、克隆、调试、自动空间推断
pub struct BaseAccount {
    /// 账户版本（支持升级与兼容性校验）
    pub version: ProgramVersion, // 账户版本号，便于升级和兼容性校验
    /// 账户权限（authority）
    pub authority: Pubkey, // 账户操作权限公钥
    /// 账户是否激活
    pub is_active: bool, // 激活状态，便于生命周期管理
    /// 账户是否暂停
    pub is_paused: bool, // 暂停状态，便于风控和合规
    /// 创建时间戳
    pub created_at: i64, // 账户创建时间，Unix时间戳
    /// 最后更新时间戳
    pub updated_at: i64, // 账户最后更新时间，Unix时间戳
    /// PDA bump seed
    pub bump: u8, // PDA bump种子，确保PDA唯一性
}

impl BaseAccount {
    /// 初始化基础账户
    /// - authority: 权限公钥
    /// - bump: PDA bump种子
    /// - 返回初始化后的BaseAccount
    pub fn new(authority: Pubkey, bump: u8) -> anchor_lang::Result<Self> {
        let current_time = Clock::get()?.unix_timestamp; // 获取当前链上时间戳
        Ok(Self {
            version: CURRENT_VERSION, // 当前合约版本
            authority, // 权限公钥
            is_active: true, // 默认激活
            is_paused: false, // 默认未暂停
            created_at: current_time, // 创建时间
            updated_at: current_time, // 更新时间
            bump, // PDA bump
        })
    }

    /// 更新时间戳
    /// - 更新updated_at为当前链上时间
    pub fn touch(&mut self) -> anchor_lang::Result<()> {
        self.updated_at = Clock::get()?.unix_timestamp; // 设置为当前时间
        Ok(())
    }
}

/// 实现 Validatable trait，统一参数合法性校验
impl crate::core::traits::Validatable for BaseAccount {
    fn validate(&self) -> anchor_lang::Result<()> {
        if self.authority == Pubkey::default() { // 权限公钥不能为默认空值
            return Err(ProgramError::InvalidStrategyParameters.into()); // 返回参数错误
        }
        Ok(())
    }
}

/// 实现 Pausable trait，支持账户暂停/恢复
impl crate::core::traits::Pausable for BaseAccount {
    fn is_paused(&self) -> bool {
        self.is_paused // 返回暂停状态
    }
    fn pause(&mut self) -> anchor_lang::Result<()> {
        self.is_paused = true; // 设置为暂停
        self.touch() // 更新时间戳
    }
    fn unpause(&mut self) -> anchor_lang::Result<()> {
        self.is_paused = false; // 取消暂停
        self.touch() // 更新时间戳
    }
    fn resume(&mut self) -> anchor_lang::Result<()> {
        self.unpause() // 恢复即为取消暂停
    }
}

/// 实现 Activatable trait，支持账户激活/失效
impl crate::core::traits::Activatable for BaseAccount {
    fn is_active(&self) -> bool {
        self.is_active // 返回激活状态
    }
    fn activate(&mut self) -> anchor_lang::Result<()> {
        self.is_active = true; // 设置为激活
        self.touch() // 更新时间戳
    }
    fn deactivate(&mut self) -> anchor_lang::Result<()> {
        self.is_active = false; // 设置为失效
        self.touch() // 更新时间戳
    }
}

/// 实现 Authorizable trait，支持权限转移
impl crate::core::traits::Authorizable for BaseAccount {
    fn authority(&self) -> Pubkey {
        self.authority // 返回当前权限公钥
    }
    fn transfer_authority(&mut self, new_authority: Pubkey) -> anchor_lang::Result<()> {
        self.authority = new_authority; // 设置新权限
        self.touch()?; // 更新时间戳
        Ok(())
    }
}

/// 实现 Versioned trait，支持账户版本管理
impl crate::version::Versioned for BaseAccount {
    fn version(&self) -> ProgramVersion {
        self.version // 返回当前版本
    }
    fn set_version(&mut self, version: ProgramVersion) {
        self.version = version; // 设置新版本
        if let Ok(clock) = Clock::get() {
            self.updated_at = clock.unix_timestamp; // 更新时间戳
        }
    }
}

/// 执行统计结构体
/// - 记录执行次数、成功/失败、gas、平均耗时等
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default, InitSpace)] // 支持Anchor序列化、克隆、调试、自动空间推断
pub struct ExecutionStats {
    /// 总执行次数
    pub total_executions: u64, // 所有执行次数
    /// 成功次数
    pub successful_executions: u64, // 成功执行次数
    /// 失败次数
    pub failed_executions: u64, // 失败执行次数
    /// 总gas消耗
    pub total_gas_used: u64, // 累计gas消耗
    /// 平均执行耗时（毫秒）
    pub avg_execution_time_ms: u64, // 平均耗时，单位毫秒
    /// 最后执行时间戳
    pub last_execution: i64, // 最后一次执行时间
}

impl ExecutionStats {
    /// 记录一次成功执行
    /// - gas_used: 本次消耗gas
    /// - execution_time_ms: 本次耗时
    pub fn record_success(&mut self, gas_used: u64, execution_time_ms: u64) -> anchor_lang::Result<()> {
        self.total_executions += 1; // 总次数+1
        self.successful_executions += 1; // 成功次数+1
        self.total_gas_used += gas_used; // 累计gas
        if self.total_executions > 0 {
            self.avg_execution_time_ms = ((self.avg_execution_time_ms * (self.total_executions - 1)) + execution_time_ms) / self.total_executions; // 更新平均耗时
        }
        self.last_execution = Clock::get()?.unix_timestamp; // 更新时间戳
        Ok(())
    }
    /// 记录一次失败执行
    pub fn record_failure(&mut self) -> anchor_lang::Result<()> {
        self.total_executions += 1; // 总次数+1
        self.failed_executions += 1; // 失败次数+1
        self.last_execution = Clock::get()?.unix_timestamp; // 更新时间戳
        Ok(())
    }
    /// 获取成功率（基点）
    pub fn success_rate_bps(&self) -> u64 {
        if self.total_executions == 0 {
            return 0; // 避免除零
        }
        (self.successful_executions * 10_000) / self.total_executions // 成功率，基点制
    }
    /// 获取平均每次gas消耗
    pub fn avg_gas_per_execution(&self) -> u64 {
        if self.total_executions == 0 {
            return 0; // 避免除零
        }
        self.total_gas_used / self.total_executions // 平均gas
    }
}

/// 账户初始化辅助器
pub struct AccountInitializer; // 工具结构体，无状态

impl AccountInitializer {
    /// 初始化基础账户
    pub fn init_base_account(authority: Pubkey, bump: u8) -> anchor_lang::Result<BaseAccount> {
        BaseAccount::new(authority, bump) // 调用BaseAccount构造
    }
    /// 校验初始化参数
    pub fn validate_init_params(authority: &Pubkey, bump: u8) -> anchor_lang::Result<()> {
        if *authority == Pubkey::default() { // 权限不能为空
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if bump == 0 { // bump不能为0
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 通用账户空间计算工具
pub struct SpaceCalculator; // 工具结构体，无状态

impl SpaceCalculator {
    /// 计算Pubkey向量所需空间
    pub fn pubkey_vec_space(max_items: usize) -> usize {
        4 + (32 * max_items) // 4字节长度 + 每个pubkey 32字节
    }
    /// 计算u64向量所需空间
    pub fn u64_vec_space(max_items: usize) -> usize {
        4 + (8 * max_items) // 4字节长度 + 每个u64 8字节
    }
    /// 计算字节向量所需空间
    pub fn bytes_vec_space(max_bytes: usize) -> usize {
        4 + max_bytes // 4字节长度 + 最大字节数
    }
    /// 计算字符串所需空间
    pub fn string_space(max_length: usize) -> usize {
        4 + max_length // 4字节长度 + 最大字符串长度
    }
    /// 计算Option类型所需空间
    pub fn option_space(inner_space: usize) -> usize {
        1 + inner_space // 1字节Some/None + 内部类型空间
    }
}

// ========================= 单元测试 =========================
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_price_feed_validate() {
        let pf = PriceFeed { mint: Pubkey::default(), price: 100 };
        assert!(pf.validate().is_err()); // mint为默认值应校验失败
        let pf2 = PriceFeed { mint: Pubkey::new_unique(), price: 100 };
        assert!(pf2.validate().is_ok()); // mint有效应校验通过
    }

    #[test]
    fn test_base_account_lifecycle() {
        let authority = Pubkey::new_unique(); // 随机权限公钥
        let bump = 42;
        let mut acc = BaseAccount::new(authority, bump).unwrap();
        assert!(acc.is_active); // 初始应激活
        assert!(!acc.is_paused); // 初始应未暂停
        acc.pause().unwrap();
        assert!(acc.is_paused); // 应暂停
        acc.unpause().unwrap();
        assert!(!acc.is_paused); // 应恢复
        acc.deactivate().unwrap();
        assert!(!acc.is_active); // 应失效
        acc.activate().unwrap();
        assert!(acc.is_active); // 应激活
    }

    #[test]
    fn test_execution_stats() {
        let mut stats = ExecutionStats::default();
        stats.record_success(100, 10).unwrap();
        stats.record_failure().unwrap();
        assert_eq!(stats.total_executions, 2); // 总次数应为2
        assert_eq!(stats.successful_executions, 1); // 成功1次
        assert_eq!(stats.failed_executions, 1); // 失败1次
        assert!(stats.avg_execution_time_ms > 0); // 平均耗时应大于0
    }
}
