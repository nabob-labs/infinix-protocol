// ========================= 策略状态结构体与逻辑实现 =========================
// 本模块为权重策略、再平衡策略等提供链上状态管理、参数、执行、AI/外部信号、权限、激活、暂停、版本、校验等，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、错误、测试等均有详细注释。

use anchor_lang::prelude::*;
use crate::state::common::*;
use crate::core::traits::*;
use anchor_lang::prelude::ProgramError;
use crate::version::{ProgramVersion, Versioned};
use crate::strategies::{WeightStrategyType, RebalancingStrategyType};

/// 权重策略账户
#[account]
#[derive(Debug, InitSpace, PartialEq, Eq)]
pub struct WeightStrategy {
    /// 通用账户基础信息
    pub base: BaseAccount,
    /// 工厂地址
    pub factory: Pubkey,
    /// 策略类型
    pub strategy_type: WeightStrategyType,
    /// 策略参数（序列化）
    #[max_len(256)]
    pub parameters: Vec<u8>,
    /// 资产mint列表
    #[max_len(16)]
    pub token_mints: Vec<Pubkey>,
    /// 当前权重
    #[max_len(16)]
    pub current_weights: Vec<u64>,
    /// 上次计算时间
    pub last_calculated: i64,
    /// 执行统计
    pub execution_stats: ExecutionStats,
    /// AI权重建议
    #[max_len(16)]
    pub ai_weights: Option<Vec<u64>>,
    /// 外部信号
    #[max_len(16)]
    pub external_signals: Option<Vec<u64>>,
}

impl WeightStrategy {
    /// 初始化策略
    pub fn initialize(&mut self, authority: Pubkey, factory: Pubkey, strategy_type: WeightStrategyType, parameters: Vec<u8>, token_mints: Vec<Pubkey>, bump: u8) -> anchor_lang::Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.factory = factory;
        self.strategy_type = strategy_type;
        self.parameters = parameters;
        self.token_mints = token_mints.clone();
        self.current_weights = vec![0; token_mints.len()];
        self.last_calculated = 0;
        self.execution_stats = ExecutionStats::default();
        self.ai_weights = None;
        self.external_signals = None;
        Ok(())
    }
    /// 校验是否可执行
    pub fn validate_can_execute(&self) -> anchor_lang::Result<()> {
        if !self.base.is_active { return Err(ProgramError::StrategyPaused.into()); }
        if self.base.is_paused { return Err(ProgramError::StrategyPaused.into()); }
        if self.token_mints.is_empty() { return Err(ProgramError::InvalidTokenCount.into()); }
        Ok(())
    }
    /// 更新权重（多因子聚合）
    pub fn update_weights(&mut self, new_weights: Vec<u64>, ai_weights: Option<Vec<u64>>, external_signals: Option<Vec<u64>>) -> anchor_lang::Result<()> {
        if new_weights.len() != self.token_mints.len() {
            return Err(ProgramError::InvalidTokenCount.into());
        }
        let total: u64 = new_weights.iter().sum();
        if total != 10_000 {
            return Err(ProgramError::InvalidWeightSum.into());
        }
        let final_weights = if let Some(ai) = &ai_weights {
            ai.iter().zip(new_weights.iter()).map(|(a, n)| ((*a + *n) / 2)).collect()
        } else {
            new_weights.clone()
        };
        self.current_weights = final_weights;
        self.last_calculated = Clock::get()?.unix_timestamp;
        self.base.touch()?;
        self.ai_weights = ai_weights;
        self.external_signals = external_signals;
        Ok(())
    }
}

/// 校验 trait 实现
impl Validatable for WeightStrategy {
    fn validate(&self) -> anchor_lang::Result<()> {
        self.base.validate()?;
        if self.factory == Pubkey::default() {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.token_mints.is_empty() || self.token_mints.len() > 16 {
            return Err(ProgramError::InvalidTokenCount.into());
        }
        if self.parameters.len() > 256 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        let mut seen = std::collections::HashSet::new();
        for mint in &self.token_mints {
            if !seen.insert(*mint) {
                return Err(ProgramError::InvalidStrategyParameters.into());
            }
        }
        Ok(())
    }
}

/// 权限 trait 实现
impl Authorizable for WeightStrategy {
    fn authority(&self) -> Pubkey { self.base.authority }
    fn transfer_authority(&mut self, new_authority: Pubkey) -> anchor_lang::Result<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

/// 暂停 trait 实现
impl Pausable for WeightStrategy {
    fn is_paused(&self) -> bool { self.base.is_paused }
    fn pause(&mut self) -> anchor_lang::Result<()> { self.base.pause() }
    fn unpause(&mut self) -> anchor_lang::Result<()> { self.base.unpause() }
    fn resume(&mut self) -> anchor_lang::Result<()> { self.unpause() }
}

/// 激活 trait 实现
impl Activatable for WeightStrategy {
    fn is_active(&self) -> bool { self.base.is_active }
    fn activate(&mut self) -> anchor_lang::Result<()> { self.base.activate() }
    fn deactivate(&mut self) -> anchor_lang::Result<()> { self.base.deactivate() }
}

/// 版本 trait 实现
impl Versioned for WeightStrategy {
    fn version(&self) -> ProgramVersion { self.base.version }
    fn set_version(&mut self, version: ProgramVersion) { self.base.set_version(version); }
}

// ========================= 再平衡策略 =========================

/// 再平衡策略账户
#[account]
#[derive(Debug, InitSpace, PartialEq, Eq)]
pub struct RebalancingStrategy {
    /// 通用账户基础信息
    pub base: BaseAccount,
    /// 工厂地址
    pub factory: Pubkey,
    /// 关联权重策略
    pub weight_strategy: Pubkey,
    /// 策略类型
    pub strategy_type: RebalancingStrategyType,
    /// 策略参数（序列化）
    #[max_len(256)]
    pub parameters: Vec<u8>,
    /// 再平衡阈值（bps）
    pub rebalancing_threshold: u64,
    /// 最小再平衡间隔（秒）
    pub min_rebalance_interval: u64,
    /// 最大滑点容忍
    pub max_slippage: u64,
    /// 上次再平衡时间
    pub last_rebalanced: i64,
    /// 执行统计
    pub execution_stats: ExecutionStats,
    /// AI信号
    #[max_len(16)]
    pub ai_signals: Option<Vec<u64>>,
    /// 外部信号
    #[max_len(16)]
    pub external_signals: Option<Vec<u64>>,
}

impl RebalancingStrategy {
    /// 初始化策略
    pub fn initialize(&mut self, authority: Pubkey, factory: Pubkey, weight_strategy: Pubkey, strategy_type: RebalancingStrategyType, parameters: Vec<u8>, rebalancing_threshold: u64, min_rebalance_interval: u64, max_slippage: u64, bump: u8) -> anchor_lang::Result<()> {
        self.base = BaseAccount::new(authority, bump)?;
        self.factory = factory;
        self.weight_strategy = weight_strategy;
        self.strategy_type = strategy_type;
        self.parameters = parameters;
        self.rebalancing_threshold = rebalancing_threshold;
        self.min_rebalance_interval = min_rebalance_interval;
        self.max_slippage = max_slippage;
        self.last_rebalanced = 0;
        self.execution_stats = ExecutionStats::default();
        self.ai_signals = None;
        self.external_signals = None;
        Ok(())
    }
    /// 校验是否可再平衡（多因子聚合）
    pub fn can_rebalance(&self, ai_signals: Option<Vec<u64>>, external_signals: Option<Vec<u64>>) -> anchor_lang::Result<bool> {
        if !self.base.is_active || self.base.is_paused { return Ok(false); }
        let current_time = Clock::get()?.unix_timestamp;
        let time_since_last = current_time - self.last_rebalanced;
        let ai_factor = ai_signals.as_ref().and_then(|v| v.first().cloned()).unwrap_or(1);
        let ext_factor = external_signals.as_ref().and_then(|v| v.first().cloned()).unwrap_or(1);
        let allow = time_since_last >= self.min_rebalance_interval as i64 && ai_factor > 0 && ext_factor > 0;
        Ok(allow)
    }
    /// 判断是否需要再平衡
    pub fn needs_rebalancing(&self, current_weights: &[u64], target_weights: &[u64]) -> bool {
        if current_weights.len() != target_weights.len() { return false; }
        for (current, target) in current_weights.iter().zip(target_weights.iter()) {
            let deviation = if current > target { current - target } else { target - current };
            if deviation >= self.rebalancing_threshold { return true; }
        }
        false
    }
    /// 更新再平衡时间
    pub fn update_rebalancing(&mut self) -> anchor_lang::Result<()> {
        self.last_rebalanced = Clock::get()?.unix_timestamp;
        self.base.touch()?;
        Ok(())
    }
}

/// 校验 trait 实现
impl Validatable for RebalancingStrategy {
    fn validate(&self) -> anchor_lang::Result<()> {
        self.base.validate()?;
        if self.factory == Pubkey::default() {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.weight_strategy == Pubkey::default() {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.rebalancing_threshold == 0 || self.rebalancing_threshold > 2000 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.min_rebalance_interval < 60 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.max_slippage > 1000 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        if self.parameters.len() > 256 {
            return Err(ProgramError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 权限 trait 实现
impl Authorizable for RebalancingStrategy {
    fn authority(&self) -> Pubkey { self.base.authority }
    fn transfer_authority(&mut self, new_authority: Pubkey) -> anchor_lang::Result<()> {
        self.base.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}

/// 暂停 trait 实现
impl Pausable for RebalancingStrategy {
    fn is_paused(&self) -> bool { self.base.is_paused }
    fn pause(&mut self) -> anchor_lang::Result<()> { self.base.pause() }
    fn unpause(&mut self) -> anchor_lang::Result<()> { self.base.unpause() }
    fn resume(&mut self) -> anchor_lang::Result<()> { self.unpause() }
}

/// 激活 trait 实现
impl Activatable for RebalancingStrategy {
    fn is_active(&self) -> bool { self.base.is_active }
    fn activate(&mut self) -> anchor_lang::Result<()> { self.base.activate() }
    fn deactivate(&mut self) -> anchor_lang::Result<()> { self.base.deactivate() }
}

/// 版本 trait 实现
impl Versioned for RebalancingStrategy {
    fn version(&self) -> ProgramVersion { self.base.version }
    fn set_version(&mut self, version: ProgramVersion) { self.base.set_version(version); }
}

// ========================= 单元测试 =========================
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_weight_strategy_validate() {
        let mut strategy = WeightStrategy {
            base: BaseAccount::new(Pubkey::new_unique(), 1).unwrap(),
            factory: Pubkey::new_unique(),
            strategy_type: WeightStrategyType::Equal,
            parameters: vec![1, 2, 3],
            token_mints: vec![Pubkey::new_unique()],
            current_weights: vec![10_000],
            last_calculated: 0,
            execution_stats: ExecutionStats::default(),
            ai_weights: None,
            external_signals: None,
        };
        assert!(strategy.validate().is_ok());
        strategy.token_mints = vec![];
        assert!(strategy.validate().is_err());
    }

    #[test]
    fn test_rebalancing_strategy_validate() {
        let mut strategy = RebalancingStrategy {
            base: BaseAccount::new(Pubkey::new_unique(), 1).unwrap(),
            factory: Pubkey::new_unique(),
            weight_strategy: Pubkey::new_unique(),
            strategy_type: RebalancingStrategyType::Threshold,
            parameters: vec![1, 2, 3],
            rebalancing_threshold: 100,
            min_rebalance_interval: 60,
            max_slippage: 100,
            last_rebalanced: 0,
            execution_stats: ExecutionStats::default(),
            ai_signals: None,
            external_signals: None,
        };
        assert!(strategy.validate().is_ok());
        strategy.rebalancing_threshold = 0;
        assert!(strategy.validate().is_err());
    }
}
