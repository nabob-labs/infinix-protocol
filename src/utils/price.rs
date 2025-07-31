//! Price模块 - 价格计算工具
//! 
//! 本模块提供价格计算功能，包含：
//! - 价格计算
//! - 价格验证
//! - 价格转换
//! - 价格格式化
//! 
//! 设计理念：
//! - 准确性：确保价格计算的准确性
//! - 性能：使用高效的算法
//! - 安全性：防止价格操纵
//! - 标准化：遵循价格计算标准
//! - 设计意图：极致精确、高性能、安全可靠

use anchor_lang::prelude::*;             // Anchor 预导入，包含Pubkey、Result等
// use crate::errors::strategy_error::StrategyError;
use crate::utils::math::MathOps;

/// 价格计算工具结构体，提供各种价格相关计算功能。
pub struct PriceUtils;

impl PriceUtils {
    /// 计算投资组合总价值。
    ///
    /// # 参数
    /// * `tokens` - 资产权重信息数组。
    /// * `price_feeds` - 价格数据源数组。
    /// # 返回
    /// * 总价值或错误。
    pub fn calculate_total_value(tokens: &[TokenWeight], price_feeds: &[PriceFeed]) -> anchor_lang::Result<u64> {
        // 校验数组长度一致。
        if tokens.len() != price_feeds.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        let mut total_value = 0u64;
        // 遍历每个资产计算价值。
        for (token, price_feed) in tokens.iter().zip(price_feeds.iter()) {
            // 校验价格数据源有效性。
            price_feed.validate()?;
            // 计算单个资产价值：余额 * 价格 / 精度。
            let token_value = MathOps::div(
                MathOps::mul(token.balance, price_feed.price)?,
                PRICE_PRECISION,
            )?;
            // 累加到总价值。
            total_value = MathOps::add(total_value, token_value)?;
        }
        Ok(total_value)
    }
    /// 计算交易价格冲击。
    ///
    /// # 参数
    /// * `trade_amount` - 交易金额。
    /// * `liquidity` - 流动性。
    /// * `current_price` - 当前价格。
    /// # 返回
    /// * 价格冲击（基点）或错误。
    pub fn calculate_price_impact(
        trade_amount: u64,
        liquidity: u64,
        current_price: u64,
    ) -> anchor_lang::Result<u64> {
        // 无流动性时返回最大冲击。
        if liquidity == 0 {
            return Ok(MAX_SLIPPAGE_BPS);
        }
        // 简化价格冲击计算：冲击 = (交易金额 / 流动性) * 冲击因子。
        let impact_factor = 5000u64; // 50% 基础冲击因子
        let raw_impact = MathOps::div(MathOps::mul(trade_amount, impact_factor)?, liquidity)?;
        // 限制在最大滑点范围内。
        Ok(raw_impact.min(MAX_SLIPPAGE_BPS))
    }
    /// 计算 TWAP（时间加权平均价格）。
    ///
    /// # 参数
    /// * `prices` - 价格数组。
    /// * `time_weights` - 时间权重数组。
    /// # 返回
    /// * TWAP 或错误。
    pub fn calculate_twap(prices: &[u64], time_weights: &[u64]) -> anchor_lang::Result<u64> {
        // 校验数组非空且长度一致。
        if prices.is_empty() || prices.len() != time_weights.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 使用数学工具计算加权平均。
        MathOps::weighted_average(prices, time_weights)
    }
    /// 计算 VWAP（成交量加权平均价格）。
    ///
    /// # 参数
    /// * `prices` - 价格数组。
    /// * `volumes` - 成交量数组。
    /// # 返回
    /// * VWAP 或错误。
    pub fn calculate_vwap(prices: &[u64], volumes: &[u64]) -> anchor_lang::Result<u64> {
        // 校验数组非空且长度一致。
        if prices.is_empty() || prices.len() != volumes.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 使用数学工具计算加权平均。
        MathOps::weighted_average(prices, volumes)
    }
    /// 计算价格波动率。
    ///
    /// # 参数
    /// * `prices` - 价格数组。
    /// # 返回
    /// * 波动率或错误。
    pub fn calculate_volatility(prices: &[u64]) -> anchor_lang::Result<u64> {
        // 至少需要 2 个价格点计算波动率。
        if prices.len() < 2 {
            return Ok(0);
        }
        // 计算均值。
        let sum: u64 = prices.iter().sum();
        let mean = sum / prices.len() as u64;
        // 计算方差。
        let variance_sum: u64 = prices
            .iter()
            .map(|&price| {
                let diff = if price > mean {
                    price - mean
                } else {
                    mean - price
                };
                diff * diff
            })
            .sum();
        let variance = variance_sum / prices.len() as u64;
        // 返回标准差（方差的平方根）。
        Ok(MathOps::sqrt(variance))
    }
    /// 计算价格变化百分比。
    ///
    /// # 参数
    /// * `old_price` - 旧价格。
    /// * `new_price` - 新价格。
    /// # 返回
    /// * 变化百分比（基点）或错误。
    pub fn calculate_price_change(old_price: u64, new_price: u64) -> anchor_lang::Result<i64> {
        // 避免除零错误。
        if old_price == 0 {
            return Ok(0);
        }
        // 计算价格变化绝对值。
        let change = if new_price >= old_price {
            MathOps::mul(new_price - old_price, BASIS_POINTS_MAX)?
        } else {
            MathOps::mul(old_price - new_price, BASIS_POINTS_MAX)?
        };
        // 计算百分比。
        let percentage = MathOps::div(change, old_price)? as i64;
        // 返回带符号的百分比。
        Ok(if new_price >= old_price {
            percentage
        } else {
            -percentage
        })
    }
    /// 计算移动平均。
    ///
    /// # 参数
    /// * `prices` - 价格数组。
    /// * `window` - 移动窗口大小。
    /// # 返回
    /// * 移动平均或错误。
    pub fn calculate_moving_average(prices: &[u64], window: usize) -> anchor_lang::Result<u64> {
        // 校验参数有效性。
        if prices.is_empty() || window == 0 || window > prices.len() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        // 计算最近 window 个价格的平均值。
        let sum: u64 = prices.iter().rev().take(window).sum();
        Ok(sum / window as u64)
    }
    /// 校验价格数据源有效性。
    ///
    /// # 参数
    /// * `price_feed` - 价格数据源。
    /// # 返回
    /// * 校验结果。
    pub fn validate_price_feed(price_feed: &PriceFeed) -> anchor_lang::Result<()> {
        // 检查价格是否有效。
        if price_feed.price == 0 {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }
        // 检查价格数据是否过期。
        let current_time = Clock::get()?.unix_timestamp;
        if current_time - price_feed.last_updated > PRICE_FEED_STALENESS_THRESHOLD {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }
        // 检查价格数据是否标记为有效。
        if !price_feed.is_valid {
            return Err(StrategyError::PriceFeedUnavailable.into());
        }
        Ok(())
    }
}

/// 资产权重结构体，用于价值计算。
#[derive(Debug, Clone)]
pub struct TokenWeight {
    pub mint: Pubkey,           // 资产 mint 地址
    pub current_weight: u64,    // 当前权重（基点）
    pub target_weight: u64,     // 目标权重（基点）
    pub balance: u64,           // 余额
    pub price: u64,             // 价格
}

/// 再平衡操作结构体。
#[derive(Debug, Clone)]
pub struct RebalanceAction {
    pub token_mint: Pubkey,     // 资产 mint 地址
    pub action_type: u8,        // 操作类型（0 = 买入，1 = 卖出）
    pub amount: u64,            // 操作金额
    pub price_impact: u64,      // 价格冲击（基点）
}

/// 为价格数据源实现校验 trait。
impl PriceFeed {
    /// 校验价格数据源有效性。
    pub fn validate(&self) -> anchor_lang::Result<()> {
        PriceUtils::validate_price_feed(self)
    }
}
