// ========================= 篮子与指数代币统一状态实现 =========================
// 本模块为篮子和指数代币提供统一状态结构体、核心业务字段和操作，
// 每个 struct、trait、impl、方法、参数、用途、边界、Anchor 相关点、事件、错误、测试等均有详细注释。

use anchor_lang::prelude::*;
use crate::core::*;
use crate::state::common::*;
use crate::errors::basket_error::BasketError;

/// 篮子/指数代币统一状态结构体
/// - 记录篮子资产、权重、供应、权限、费用、状态、统计、风险等
/// - 适用于所有 Anchor 账户，支持升级、权限、激活/暂停、再平衡等
#[account]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace, PartialEq, Eq)]
pub struct BasketIndexState {
    /// 通用账户基础信息
    pub base: BaseAccount,
    /// 篮子唯一ID
    pub id: u64,
    /// 资产类型（如ETF、指数、加密货币、RWA等）
    pub asset_type: crate::core::types::AssetType, // 新增字段，标识资产类型，便于多资产类型融合与扩展
    /// 资产成分（最大16种）
    pub composition: Vec<BasketConstituent>,
    /// 各资产权重（bps，最大16项）
    pub weights: Vec<u64>,
    /// 当前总价值（USDC计价）
    pub total_value: u64,
    /// 当前总供应量
    pub total_supply: u64,
    /// 权限账户
    pub authority: Pubkey,
    /// 管理员账户（可选）
    pub manager: Option<Pubkey>,
    /// 费用收集账户
    pub fee_collector: Pubkey,
    /// 创建费率（bps）
    pub creation_fee_bps: u16,
    /// 赎回费率（bps）
    pub redemption_fee_bps: u16,
    /// 当前状态
    pub status: BasketStatus,
    /// 是否激活
    pub is_active: bool,
    /// 是否暂停
    pub is_paused: bool,
    /// 是否启用再平衡
    pub enable_rebalancing: bool,
    /// 上次再平衡时间
    pub last_rebalanced: i64,
    /// 创建时间
    pub created_at: i64,
    /// 最后更新时间
    pub updated_at: i64,
    /// 执行统计
    pub execution_stats: ExecutionStats,
    /// 风险指标
    pub risk_metrics: Option<RiskMetrics>,
    /// AI信号
    pub ai_signals: Option<Vec<u64>>,
    /// 外部信号
    pub external_signals: Option<Vec<u64>>,
    /// PDA bump
    pub bump: u8,
}

impl BasketIndexState {
    /// 初始化篮子状态
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        manager: Option<Pubkey>,
        id: u64,
        composition: Vec<BasketConstituent>,
        weights: Vec<u64>,
        fee_collector: Pubkey,
        creation_fee_bps: u16,
        redemption_fee_bps: u16,
        enable_rebalancing: bool,
        bump: u8,
    ) {
        let now = Clock::get().unwrap().unix_timestamp;
        self.id = id;
        self.composition = composition;
        self.weights = weights;
        self.total_value = 0;
        self.total_supply = 0;
        self.authority = authority;
        self.manager = manager;
        self.fee_collector = fee_collector;
        self.creation_fee_bps = creation_fee_bps;
        self.redemption_fee_bps = redemption_fee_bps;
        self.status = BasketStatus::Active;
        self.is_active = true;
        self.is_paused = false;
        self.enable_rebalancing = enable_rebalancing;
        self.last_rebalanced = now;
        self.created_at = now;
        self.updated_at = now;
        self.execution_stats = ExecutionStats::default();
        self.risk_metrics = None;
        self.ai_signals = None;
        self.external_signals = None;
        self.bump = bump;
    }
    /// 铸造新代币
    pub fn mint_tokens(&mut self, amount: u64) -> Result<()> {
        self.total_supply = self.total_supply.checked_add(amount).ok_or(BasketError::Overflow)?;
        Ok(())
    }
    /// 销毁代币
    pub fn burn_tokens(&mut self, amount: u64) -> Result<()> {
        require!(self.total_supply >= amount, BasketError::InsufficientValue);
        self.total_supply -= amount;
        Ok(())
    }
}

/// 实现 NAV 计算 trait
impl NavCalculable for BasketIndexState {
    fn calculate_nav(&self, price_feeds: &[PriceFeed]) -> Result<u64> {
        let mut total_value = 0u64;
        for constituent in &self.composition {
            if let Some(price_feed) = price_feeds.iter().find(|pf| pf.mint == constituent.token_mint) {
                price_feed.validate()?;
                let constituent_value = constituent.balance.checked_mul(price_feed.price).ok_or(BasketError::Overflow)?
                    .checked_div(PRICE_PRECISION).ok_or(BasketError::Overflow)?;
                total_value = total_value.checked_add(constituent_value).ok_or(BasketError::Overflow)?;
            }
        }
        Ok(total_value)
    }
}

/// 实现费用管理 trait
impl FeeManageable for BasketIndexState {
    fn collect_fees(&mut self, amount: u64) -> Result<()> {
        self.total_value = self.total_value.checked_add(amount).ok_or(BasketError::Overflow)?;
        Ok(())
    }
}

/// 实现再平衡 trait
impl Rebalancable for BasketIndexState {
    fn rebalance(&mut self, new_weights: Vec<u64>) -> Result<()> {
        if new_weights.len() != self.weights.len() {
            return Err(BasketError::InvalidTokenCount.into());
        }
        let total_weight: u64 = new_weights.iter().sum();
        if total_weight != 10_000 {
            return Err(BasketError::InvalidWeightSum.into());
        }
        self.weights = new_weights;
        self.last_rebalanced = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

/// 实现操作统计 trait
impl OperationStats for BasketIndexState {
    fn record_operation(&mut self) {
        self.execution_stats.total_executions += 1;
        self.execution_stats.last_execution = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
    }
}

/// 实现激活/暂停/权限/版本 trait
impl crate::core::traits::Activatable for BasketIndexState {
    fn is_active(&self) -> bool { self.is_active }
    fn activate(&mut self) -> Result<()> { self.is_active = true; self.base.touch() }
    fn deactivate(&mut self) -> Result<()> { self.is_active = false; self.base.touch() }
}
impl crate::core::traits::Pausable for BasketIndexState {
    fn is_paused(&self) -> bool { self.is_paused }
    fn pause(&mut self) -> Result<()> { self.is_paused = true; self.base.touch() }
    fn unpause(&mut self) -> Result<()> { self.is_paused = false; self.base.touch() }
    fn resume(&mut self) -> Result<()> { self.unpause() }
}
impl crate::core::traits::Authorizable for BasketIndexState {
    fn authority(&self) -> Pubkey { self.authority }
    fn transfer_authority(&mut self, new_authority: Pubkey) -> Result<()> {
        self.authority = new_authority;
        self.base.touch()?;
        Ok(())
    }
}
impl crate::version::Versioned for BasketIndexState {
    fn version(&self) -> ProgramVersion { self.base.version }
    fn set_version(&mut self, version: ProgramVersion) { self.base.set_version(version); }
}

/// 实现统一校验 trait
impl crate::core::traits::Validatable for BasketIndexState {
    fn validate(&self) -> Result<()> {
        self.base.validate()?;
        if self.fee_collector == Pubkey::default() {
            return Err(BasketError::InvalidAssets.into());
        }
        if self.composition.is_empty() {
            return Err(BasketError::InvalidAssets.into());
        }
        let total_weight: u64 = self.weights.iter().sum();
        if total_weight != 10_000 {
            return Err(BasketError::InvalidWeightSum.into());
        }
        Ok(())
    }
}

/// 篮子状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum BasketStatus {
    /// 活跃
    Active,
    /// 冻结
    Frozen,
    /// 待处理
    Pending,
    /// 已弃用
    Deprecated,
}

/// 篮子资产成分
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub struct BasketConstituent {
    /// 资产mint
    pub token_mint: Pubkey,
    /// 当前余额
    pub balance: u64,
    /// 权重
    pub weight: u64,
}

/// 风险指标
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace, Default)]
pub struct RiskMetrics {
    /// 风险评分
    pub risk_score: u32,
    /// 最大回撤
    pub max_drawdown: u64,
}

// ========================= 事件定义 =========================
#[event]
pub struct BasketCreated {
    pub basket_id: u64,
    pub assets: Vec<Pubkey>,
    pub weights: Vec<u64>,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct BasketRebalanced {
    pub basket_id: u64,
    pub new_weights: Vec<u64>,
    pub timestamp: i64,
}

// ========================= 单元测试 =========================
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_basket_validate() {
        let base = BaseAccount::new(Pubkey::new_unique(), 1).unwrap();
        let mut basket = BasketIndexState {
            base,
            id: 1,
            composition: vec![BasketConstituent { token_mint: Pubkey::new_unique(), balance: 100, weight: 10_000 }],
            weights: vec![10_000],
            total_value: 0,
            total_supply: 0,
            authority: Pubkey::new_unique(),
            manager: None,
            fee_collector: Pubkey::new_unique(),
            creation_fee_bps: 10,
            redemption_fee_bps: 10,
            status: BasketStatus::Active,
            is_active: true,
            is_paused: false,
            enable_rebalancing: true,
            last_rebalanced: 0,
            created_at: 0,
            updated_at: 0,
            execution_stats: ExecutionStats::default(),
            risk_metrics: None,
            ai_signals: None,
            external_signals: None,
            bump: 1,
        };
        assert!(basket.validate().is_ok());
        basket.weights = vec![5_000];
        assert!(basket.validate().is_err());
    }
}
