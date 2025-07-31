//!
//! Asset Validation Utilities
//!
//! 本模块实现资产相关的合规性校验工具，涵盖资产唯一性、有效性、权重、价格、权限等多维度校验，确保资产管理安全、合规、可追溯。

// 引入核心模块和 Anchor 依赖。
use anchor_lang::prelude::*;

/// 资产校验工具结构体。
pub struct AssetValidationUtils;

impl AssetValidationUtils {
    /// 校验资产地址是否有效。
    pub fn validate_asset_pubkey(pubkey: &Pubkey) -> bool {
        // 检查 pubkey 是否为全零（无效地址）
        !pubkey.to_bytes().iter().all(|&b| b == 0)
    }
    /// 校验资产权重是否在合理范围。
    pub fn validate_asset_weight(weight: u64) -> bool {
        // 权重应在 0 ~ MAX_TOKEN_WEIGHT_BPS 之间
        weight <= MAX_TOKEN_WEIGHT_BPS
    }
    /// 校验资产价格是否有效。
    pub fn validate_asset_price(price: u64) -> bool {
        // 价格应大于 0
        price > 0
    }
    /// 校验资产名称是否合法。
    pub fn validate_asset_name(name: &str) -> bool {
        // 名称长度应在 1~32 之间
        let len = name.len();
        len > 0 && len <= 32
    }
    /// 校验资产精度是否合法。
    pub fn validate_asset_decimals(decimals: u8) -> bool {
        // 精度应在 0~18 之间
        decimals <= 18
    }
    /// 校验资产是否唯一（无重复）。
    pub fn validate_no_duplicate_assets(pubkeys: &[Pubkey]) -> bool {
        // 使用 HashSet 检查去重
        let mut set = std::collections::HashSet::new();
        for pubkey in pubkeys {
            if !set.insert(*pubkey) {
                return false;
            }
        }
        true
    }
    /// 校验资产数组长度是否在范围内。
    pub fn validate_asset_count(count: usize) -> bool {
        // 资产数量应在 1~MAX_TOKENS 之间
        count > 0 && count <= MAX_TOKENS
    }
    /// 校验资产权重数组总和是否为 100%。
    pub fn validate_weights_sum(weights: &[u64]) -> bool {
        let sum: u128 = weights.iter().map(|&w| w as u128).sum();
        sum == BASIS_POINTS_MAX as u128
    }
    /// 校验资产价格数组长度与资产数量一致。
    pub fn validate_prices_length(prices: &[u64], assets: &[Pubkey]) -> bool {
        prices.len() == assets.len()
    }
    /// 校验资产权限是否有效。
    pub fn validate_asset_authority(authority: &Pubkey, expected: &Pubkey) -> bool {
        authority == expected
    }
    /// 校验资产最小金额。
    pub fn validate_minimum_amount(amount: u64) -> bool {
        amount >= 1
    }
    /// 校验资产价格新鲜度。
    pub fn validate_price_freshness(timestamp: i64, max_age_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - timestamp <= max_age_seconds
    }
}
