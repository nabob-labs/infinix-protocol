//!
//! Index Token Validation Utilities
//!
//! 本模块实现指数代币相关的合规性校验工具，涵盖代币唯一性、参数、权重、权限、价格等多维度校验，确保指数代币管理安全、合规、可追溯。

// 引入核心模块和 Anchor 依赖。
use anchor_lang::prelude::*;

/// 指数代币校验工具结构体。
pub struct IndexTokenValidationUtils;

impl IndexTokenValidationUtils {
    /// 校验指数代币名称是否合法。
    pub fn validate_index_token_name(name: &str) -> bool {
        // 名称长度应在 1~32 之间
        let len = name.len();
        len > 0 && len <= 32
    }
    /// 校验指数代币符号是否合法。
    pub fn validate_index_token_symbol(symbol: &str) -> bool {
        // 符号长度应在 1~10 之间
        let len = symbol.len();
        len > 0 && len <= 10
    }
    /// 校验指数代币资产数量是否在范围内。
    pub fn validate_index_token_asset_count(count: usize) -> bool {
        // 资产数量应在 1~MAX_TOKENS 之间
        count > 0 && count <= MAX_TOKENS
    }
    /// 校验指数代币权重数组总和是否为 100%。
    pub fn validate_weights_sum(weights: &[u64]) -> bool {
        let sum: u128 = weights.iter().map(|&w| w as u128).sum();
        sum == BASIS_POINTS_MAX as u128
    }
    /// 校验指数代币资产地址是否唯一。
    pub fn validate_no_duplicate_assets(pubkeys: &[Pubkey]) -> bool {
        let mut set = std::collections::HashSet::new();
        for pubkey in pubkeys {
            if !set.insert(*pubkey) {
                return false;
            }
        }
        true
    }
    /// 校验指数代币最小金额。
    pub fn validate_minimum_amount(amount: u64) -> bool {
        amount >= 1
    }
    /// 校验指数代币权限是否有效。
    pub fn validate_index_token_authority(authority: &Pubkey, expected: &Pubkey) -> bool {
        authority == expected
    }
    /// 校验指数代币价格数组长度与资产数量一致。
    pub fn validate_prices_length(prices: &[u64], assets: &[Pubkey]) -> bool {
        prices.len() == assets.len()
    }
    /// 校验指数代币价格新鲜度。
    pub fn validate_price_freshness(timestamp: i64, max_age_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - timestamp <= max_age_seconds
    }
} 