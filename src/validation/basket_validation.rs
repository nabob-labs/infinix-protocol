//!
//! Basket Validation Utilities
//!
//! 本模块实现篮子相关的合规性校验工具，涵盖篮子唯一性、资产组成、权重、阈值、权限等多维度校验，确保篮子管理安全、合规、可追溯。

// 引入核心模块和 Anchor 依赖。
use crate::core::*;
use anchor_lang::prelude::*;

/// 篮子校验工具结构体。
pub struct BasketValidationUtils;

impl BasketValidationUtils {
    /// 校验篮子名称是否合法。
    pub fn validate_basket_name(name: &str) -> bool {
        // 名称长度应在 1~32 之间
        let len = name.len();
        len > 0 && len <= 32
    }
    /// 校验篮子资产数量是否在范围内。
    pub fn validate_basket_asset_count(count: usize) -> bool {
        // 资产数量应在 1~MAX_TOKENS 之间
        count > 0 && count <= MAX_TOKENS
    }
    /// 校验篮子权重数组总和是否为 100%。
    pub fn validate_weights_sum(weights: &[u64]) -> bool {
        let sum: u128 = weights.iter().map(|&w| w as u128).sum();
        sum == BASIS_POINTS_MAX as u128
    }
    /// 校验篮子资产地址是否唯一。
    pub fn validate_no_duplicate_assets(pubkeys: &[Pubkey]) -> bool {
        let mut set = std::collections::HashSet::new();
        for pubkey in pubkeys {
            if !set.insert(*pubkey) {
                return false;
            }
        }
        true
    }
    /// 校验篮子阈值参数是否合法。
    pub fn validate_threshold(threshold_bps: u64) -> bool {
        // 阈值应在 0~MAX_REBALANCE_THRESHOLD_BPS 之间
        threshold_bps <= MAX_REBALANCE_THRESHOLD_BPS
    }
    /// 校验篮子最小金额。
    pub fn validate_minimum_amount(amount: u64) -> bool {
        amount >= 1
    }
    /// 校验篮子权限是否有效。
    pub fn validate_basket_authority(authority: &Pubkey, expected: &Pubkey) -> bool {
        authority == expected
    }
    /// 校验篮子资产价格数组长度与资产数量一致。
    pub fn validate_prices_length(prices: &[u64], assets: &[Pubkey]) -> bool {
        prices.len() == assets.len()
    }
    /// 校验篮子价格新鲜度。
    pub fn validate_price_freshness(timestamp: i64, max_age_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - timestamp <= max_age_seconds
    }
}
