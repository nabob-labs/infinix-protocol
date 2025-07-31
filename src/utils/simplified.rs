//! Simplified模块 - 简化工具
//! 
//! 本模块提供简化功能，包含：
//! - 简化计算
//! - 简化验证
//! - 简化转换
//! - 简化格式化
//! 
//! 设计理念：
//! - 简洁性：提供简洁的接口
//! - 易用性：易于使用和理解
//! - 可靠性：确保结果的可靠性
//! - 标准化：遵循标准规范
//! - 设计意图：极致简洁、易用可靠

use anchor_lang::prelude::*;             // Anchor 预导入，包含Pubkey、Result等

/// 简化数学运算工具结构体。
pub struct SimpleMath;

impl SimpleMath {
    /// 计算百分比（基点制）。
    pub fn percentage(part: u64, total: u64) -> u64 {
        if total == 0 {
            return 0;
        }
        (part * BASIS_POINTS_MAX) / total
    }
    /// 计算权重平均值。
    pub fn weighted_average(values: &[u64], weights: &[u64]) -> u64 {
        if values.is_empty() || weights.is_empty() || values.len() != weights.len() {
            return 0;
        }
        let mut weighted_sum = 0u128;
        let mut total_weight = 0u128;
        for (value, weight) in values.iter().zip(weights.iter()) {
            weighted_sum += (*value as u128) * (*weight as u128);
            total_weight += *weight as u128;
        }
        if total_weight == 0 {
            return 0;
        }
        (weighted_sum / total_weight) as u64
    }
    /// 计算简单移动平均。
    pub fn simple_moving_average(prices: &[u64], period: usize) -> u64 {
        if prices.is_empty() || period == 0 || period > prices.len() {
            return 0;
        }
        let start_idx = prices.len().saturating_sub(period);
        let sum: u128 = prices[start_idx..].iter().map(|&p| p as u128).sum();
        (sum / period as u128) as u64
    }
    /// 计算价格变化百分比。
    pub fn price_change_percentage(current: u64, previous: u64) -> i64 {
        if previous == 0 {
            return 0;
        }
        let change = current as i128 - previous as i128;
        ((change * BASIS_POINTS_MAX as i128) / previous as i128) as i64
    }
    /// 标准化权重数组。
    pub fn normalize_weights(weights: &[u64]) -> Vec<u64> {
        if weights.is_empty() {
            return vec![];
        }
        let total: u128 = weights.iter().map(|&w| w as u128).sum();
        if total == 0 {
            return vec![0; weights.len()];
        }
        weights
            .iter()
            .map(|&w| ((w as u128 * BASIS_POINTS_MAX as u128) / total) as u64)
            .collect()
    }
}

/// 简化验证工具结构体。
pub struct SimpleValidation;

impl SimpleValidation {
    /// 验证权重总和是否为 100%。
    pub fn validate_weights_sum(weights: &[u64]) -> bool {
        let sum: u128 = weights.iter().map(|&w| w as u128).sum();
        sum == BASIS_POINTS_MAX as u128
    }
    /// 验证数组长度是否在范围内。
    pub fn validate_array_length<T>(array: &[T], min: usize, max: usize) -> bool {
        let len = array.len();
        len >= min && len <= max
    }
    /// 验证数值是否在范围内。
    pub fn validate_range(value: u64, min: u64, max: u64) -> bool {
        value >= min && value <= max
    }
    /// 验证时间戳是否新鲜。
    pub fn validate_timestamp_freshness(timestamp: i64, max_age_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - timestamp <= max_age_seconds
    }
    /// 验证地址是否有效。
    pub fn validate_pubkey(pubkey: &Pubkey) -> bool {
        !pubkey.to_bytes().iter().all(|&b| b == 0)
    }
}

/// 简化格式化工具结构体。
pub struct SimpleFormat;

impl SimpleFormat {
    /// 格式化百分比显示。
    pub fn format_percentage(bps: u64) -> String {
        let percentage = bps as f64 / BASIS_POINTS_MAX as f64 * 100.0;
        format!("{:.2}%", percentage)
    }
    /// 格式化价格显示。
    pub fn format_price(price: u64, decimals: u8) -> String {
        let price_f64 = price as f64 / 10_u64.pow(decimals as u32) as f64;
        format!("{:.6}", price_f64)
    }
    /// 格式化权重显示。
    pub fn format_weights(weights: &[u64]) -> String {
        let formatted: Vec<String> = weights
            .iter()
            .map(|&w| Self::format_percentage(w))
            .collect();
        format!("[{}]", formatted.join(", "))
    }
    /// 格式化错误信息。
    pub fn format_error(error: &str, context: &str) -> String {
        format!("{}: {}", context, error)
    }
}

/// 简化计算工具结构体。
pub struct SimpleCalculator;

impl SimpleCalculator {
    /// 计算总价值。
    pub fn calculate_total_value(amounts: &[u64], prices: &[u64]) -> u64 {
        if amounts.len() != prices.len() {
            return 0;
        }
        amounts
            .iter()
            .zip(prices.iter())
            .map(|(&amount, &price)| {
                let product = (amount as u128) * (price as u128);
                (product / PRICE_PRECISION as u128) as u64
            })
            .sum()
    }
    /// 计算权重分配金额。
    pub fn calculate_weighted_amounts(
        total_amount: u64,
        weights: &[u64],
    ) -> Vec<u64> {
        if weights.is_empty() {
            return vec![];
        }
        weights
            .iter()
            .map(|&weight| {
                let weighted_amount = (total_amount as u128) * (weight as u128);
                (weighted_amount / BASIS_POINTS_MAX as u128) as u64
            })
            .collect()
    }
    /// 计算滑点影响。
    pub fn calculate_slippage_impact(
        amount: u64,
        slippage_bps: u64,
    ) -> u64 {
        (amount * slippage_bps) / BASIS_POINTS_MAX
    }
    /// 计算费用金额。
    pub fn calculate_fee_amount(amount: u64, fee_bps: u64) -> u64 {
        (amount * fee_bps) / BASIS_POINTS_MAX
    }
    /// 计算净金额（扣除费用）。
    pub fn calculate_net_amount(amount: u64, fee_bps: u64) -> u64 {
        amount.saturating_sub(Self::calculate_fee_amount(amount, fee_bps))
    }
}

/// 简化统计工具结构体。
pub struct SimpleStats;

impl SimpleStats {
    /// 计算数组最小值。
    pub fn min(values: &[u64]) -> u64 {
        values.iter().min().copied().unwrap_or(0)
    }
    /// 计算数组最大值。
    pub fn max(values: &[u64]) -> u64 {
        values.iter().max().copied().unwrap_or(0)
    }
    /// 计算数组平均值。
    pub fn mean(values: &[u64]) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let sum: u128 = values.iter().map(|&v| v as u128).sum();
        (sum / values.len() as u128) as u64
    }
    /// 计算数组总和。
    pub fn sum(values: &[u64]) -> u64 {
        values.iter().sum()
    }
    /// 计算数组长度。
    pub fn count(values: &[u64]) -> usize {
        values.len()
    }
}

/// 简化比较工具结构体。
pub struct SimpleCompare;

impl SimpleCompare {
    /// 比较两个数值，返回较大值。
    pub fn max(a: u64, b: u64) -> u64 {
        a.max(b)
    }
    /// 比较两个数值，返回较小值。
    pub fn min(a: u64, b: u64) -> u64 {
        a.min(b)
    }
    /// 检查数值是否在范围内。
    pub fn in_range(value: u64, min: u64, max: u64) -> bool {
        value >= min && value <= max
    }
    /// 将数值限制在范围内。
    pub fn clamp(value: u64, min: u64, max: u64) -> u64 {
        value.clamp(min, max)
    }
    /// 检查数组是否有序。
    pub fn is_sorted_ascending(values: &[u64]) -> bool {
        values.windows(2).all(|window| window[0] <= window[1])
    }
    /// 检查数组是否有序（降序）。
    pub fn is_sorted_descending(values: &[u64]) -> bool {
        values.windows(2).all(|window| window[0] >= window[1])
    }
}
