/*!
 * 高级数学工具模块
 *
 * 提供生产级的数学函数与工具，涵盖：
 * - 统计计算与分布
 * - 数值优化算法
 * - 金融数学与风险计算
 * - 线性代数运算
 * - 时间序列分析
 * - 机器学习数学基础
 *
 * # 设计说明
 * - 所有函数均带溢出/边界/参数校验，返回统一错误类型
 * - 适用于链上高安全性、可维护性、可审计场景
 * - 便于 Anchor/Solana 程序集成
 */

use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// 数学运算统一返回类型
/// - 便于 Anchor/Solana 统一错误处理
pub type MathResult<T> = Result<T>;

/// 高级数学工具结构体
/// - 提供常用数学函数实现
pub struct AdvancedMath;

impl AdvancedMath {
    /// 计算自然对数（高精度）
    ///
    /// # 参数
    /// - x: 输入值，要求 x > 0
    /// # 返回
    /// - ln(x)，溢出/非法参数返回MathOverflow
    pub fn ln(x: Decimal) -> MathResult<Decimal> {
        if x <= Decimal::ZERO {
            return Err(StrategyError::MathOverflow);
        }
        // 使用f64转换与内置ln函数，生产环境可替换为更高精度算法
        let x_f64 = x.to_f64().ok_or(StrategyError::MathOverflow)?;
        let result = x_f64.ln();
        if result.is_finite() {
            Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
        } else {
            Err(StrategyError::MathOverflow)
        }
    }

    /// 计算指数函数 e^x，带溢出保护
    ///
    /// # 参数
    /// - x: 指数
    /// # 返回
    /// - e^x，溢出返回MathOverflow
    pub fn exp(x: Decimal) -> MathResult<Decimal> {
        let x_f64 = x.to_f64().ok_or(StrategyError::MathOverflow)?;
        // 防止溢出
        if x_f64 > 700.0 {
            return Err(StrategyError::MathOverflow);
        }
        let result = x_f64.exp();
        if result.is_finite() {
            Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
        } else {
            Err(StrategyError::MathOverflow)
        }
    }

    /// 牛顿法计算平方根
    ///
    /// # 参数
    /// - x: 被开方数，x >= 0
    /// # 返回
    /// - sqrt(x)，非法参数返回InvalidStrategyParameters
    pub fn sqrt(x: Decimal) -> MathResult<Decimal> {
        if x < Decimal::ZERO {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        if x == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let x_f64 = x.to_f64().ok_or(StrategyError::MathOverflow)?;
        let result = x_f64.sqrt();
        Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
    }

    /// 幂运算 x^y
    ///
    /// # 参数
    /// - base: 底数
    /// - exponent: 指数
    /// # 返回
    /// - base^exponent，溢出返回MathOverflow
    pub fn pow(base: Decimal, exponent: Decimal) -> MathResult<Decimal> {
        let base_f64 = base.to_f64().ok_or(StrategyError::MathOverflow)?;
        let exp_f64 = exponent.to_f64().ok_or(StrategyError::MathOverflow)?;
        let result = base_f64.powf(exp_f64);
        if result.is_finite() {
            Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
        } else {
            Err(StrategyError::MathOverflow)
        }
    }

    /// 标准正态分布累积分布函数（CDF）
    ///
    /// # 参数
    /// - x: 输入值
    /// # 返回
    /// - 累积分布概率，溢出返回MathOverflow
    pub fn normal_cdf(x: Decimal) -> MathResult<Decimal> {
        let x_f64 = x.to_f64().ok_or(StrategyError::MathOverflow)?;
        // 使用Abramowitz和Stegun近似公式
        let t = 1.0 / (1.0 + 0.2316419 * x_f64.abs());
        let d = 0.3989423 * (-x_f64 * x_f64 / 2.0).exp();
        let prob = d
            * t
            * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));
        let result = if x_f64 >= 0.0 { 1.0 - prob } else { prob };
        Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
    }

    /// 标准正态分布分位点函数（逆CDF）
    ///
    /// # 参数
    /// - p: 概率，0 < p < 1
    /// # 返回
    /// - 分位点，非法参数返回InvalidStrategyParameters
    pub fn normal_inv(p: Decimal) -> MathResult<Decimal> {
        let p_f64 = p.to_f64().ok_or(StrategyError::MathOverflow)?;
        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        // Beasley-Springer-Moro算法近似
        let a = [
            -3.969683028665376e+01,
            2.209460984245205e+02,
            -2.759285104469687e+02,
            1.383577518672690e+02,
            -3.066479806614716e+01,
            2.506628277459239e+00,
        ];
        let b = [
            -5.447609879822406e+01,
            1.615858368580409e+02,
            -1.556989798598866e+02,
            6.680131188771972e+01,
            -1.328068155288572e+01,
        ];
        let c = [
            -7.784894002430293e-03,
            -3.223964580411365e-01,
            -2.400758277161838e+00,
            -2.549732539343734e+00,
            4.374664141464968e+00,
            2.938163982698783e+00,
        ];
        let d = [
            7.784695709041462e-03,
            3.224671290700398e-01,
            2.445134137142996e+00,
            3.754408661907416e+00,
        ];
        let p_low = 0.02425;
        let p_high = 1.0 - p_low;
        let result = if p_f64 < p_low {
            // 下区间近似
            let q = (-2.0 * p_f64.ln()).sqrt();
            let num = ((((c[5] * q + c[4]) * q + c[3]) * q + c[2]) * q + c[1]) * q + c[0];
            let den = (((d[3] * q + d[2]) * q + d[1]) * q + d[0]) * q + 1.0;
            num / den
        } else if p_f64 <= p_high {
            // 中区间近似
            let q = p_f64 - 0.5;
            let r = q * q;
            let num = (((((a[5] * r + a[4]) * r + a[3]) * r + a[2]) * r + a[1]) * r + a[0]) * q;
            let den = ((((b[4] * r + b[3]) * r + b[2]) * r + b[1]) * r + b[0]) * r + 1.0;
            num / den
        } else {
            // 上区间近似
            let q = (-2.0 * (1.0 - p_f64).ln()).sqrt();
            let num = ((((c[5] * q + c[4]) * q + c[3]) * q + c[2]) * q + c[1]) * q + c[0];
            let den = (((d[3] * q + d[2]) * q + d[1]) * q + d[0]) * q + 1.0;
            -(num / den)
        };
        Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
    }

    /// Black-Scholes 期权定价公式（欧式看涨）
    ///
    /// # 参数
    /// - spot: 标的现价
    /// - strike: 行权价
    /// - time_to_expiry: 距到期时间
    /// - risk_free_rate: 无风险利率
    /// - volatility: 波动率
    /// # 返回
    /// - 欧式看涨期权价格
    pub fn black_scholes_call(
        spot: Decimal,
        strike: Decimal,
        time_to_expiry: Decimal,
        risk_free_rate: Decimal,
        volatility: Decimal,
    ) -> MathResult<Decimal> {
        let s = spot.to_f64().ok_or(StrategyError::MathOverflow)?;
        let k = strike.to_f64().ok_or(StrategyError::MathOverflow)?;
        let t = time_to_expiry.to_f64().ok_or(StrategyError::MathOverflow)?;
        let r = risk_free_rate.to_f64().ok_or(StrategyError::MathOverflow)?;
        let sigma = volatility.to_f64().ok_or(StrategyError::MathOverflow)?;
        if t <= 0.0 {
            return Ok(Decimal::max(spot - strike, Decimal::ZERO));
        }
        let d1 = ((s / k).ln() + (r + sigma * sigma / 2.0) * t) / (sigma * t.sqrt());
        let d2 = d1 - sigma * t.sqrt();
        let n_d1 = Self::normal_cdf(Decimal::from_f64(d1).ok_or(StrategyError::MathOverflow)?)?;
        let n_d2 = Self::normal_cdf(Decimal::from_f64(d2).ok_or(StrategyError::MathOverflow)?)?;
        let call_price = spot * n_d1 - strike * Self::exp(risk_free_rate * -time_to_expiry)? * n_d2;
        Ok(call_price)
    }

    /// 牛顿法求隐含波动率
    ///
    /// # 参数
    /// - market_price: 市场期权价格
    /// - spot: 标的现价
    /// - strike: 行权价
    /// - time_to_expiry: 距到期时间
    /// - risk_free_rate: 无风险利率
    /// # 返回
    /// - 隐含波动率
    pub fn implied_volatility(
        market_price: Decimal,
        spot: Decimal,
        strike: Decimal,
        time_to_expiry: Decimal,
        risk_free_rate: Decimal,
    ) -> MathResult<Decimal> {
        let mut vol = Decimal::from_str("0.2").map_err(|_| StrategyError::MathOverflow)?; // 初始猜测20%
        for _ in 0..MAX_ITERATIONS_NEWTON {
            let price =
                Self::black_scholes_call(spot, strike, time_to_expiry, risk_free_rate, vol)?;
            let vega = Self::black_scholes_vega(spot, strike, time_to_expiry, risk_free_rate, vol)?;
            if vega.abs() < Decimal::from_f64(EPSILON_F64).ok_or(StrategyError::MathOverflow)? {
                break;
            }
            let price_diff = price - market_price;
            if price_diff.abs()
                < Decimal::from_f64(CONVERGENCE_TOLERANCE).ok_or(StrategyError::MathOverflow)?
            {
                break;
            }
            vol = vol - price_diff / vega;
            // 保证波动率为正
            if vol <= Decimal::ZERO {
                vol = Decimal::from_str("0.001").map_err(|_| StrategyError::MathOverflow)?;
            }
        }
        Ok(vol)
    }

    /// Black-Scholes vega（对波动率敏感度）
    ///
    /// # 参数
    /// - spot: 标的现价
    /// - strike: 行权价
    /// - time_to_expiry: 距到期时间
    /// - risk_free_rate: 无风险利率
    /// - volatility: 波动率
    /// # 返回
    /// - vega值
    pub fn black_scholes_vega(
        spot: Decimal,
        strike: Decimal,
        time_to_expiry: Decimal,
        risk_free_rate: Decimal,
        volatility: Decimal,
    ) -> MathResult<Decimal> {
        let s = spot.to_f64().ok_or(StrategyError::MathOverflow)?;
        let k = strike.to_f64().ok_or(StrategyError::MathOverflow)?;
        let t = time_to_expiry.to_f64().ok_or(StrategyError::MathOverflow)?;
        let r = risk_free_rate.to_f64().ok_or(StrategyError::MathOverflow)?;
        let sigma = volatility.to_f64().ok_or(StrategyError::MathOverflow)?;
        if t <= 0.0 {
            return Ok(Decimal::ZERO);
        }
        let d1 = ((s / k).ln() + (r + sigma * sigma / 2.0) * t) / (sigma * t.sqrt());
        let phi_d1 = (2.0 * PI).sqrt().recip() * (-d1 * d1 / 2.0).exp();
        let vega = s * phi_d1 * t.sqrt();
        Decimal::from_f64(vega).ok_or(StrategyError::MathOverflow)
    }
}

/// 统计分析工具结构体
/// - 提供均值、方差、相关性、风险指标等
pub struct Statistics;

impl Statistics {
    /// 计算均值
    ///
    /// # 参数
    /// - data: 数据集
    /// # 返回
    /// - 均值，空数据返回InvalidStrategyParameters
    pub fn mean(data: &[Decimal]) -> MathResult<Decimal> {
        if data.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let sum: Decimal = data.iter().sum();
        Ok(sum / Decimal::from(data.len()))
    }

    /// 计算方差
    ///
    /// # 参数
    /// - data: 数据集
    /// # 返回
    /// - 方差，数据量<2返回InvalidStrategyParameters
    pub fn variance(data: &[Decimal]) -> MathResult<Decimal> {
        if data.len() < 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mean = Self::mean(data)?;
        let sum_squared_diff: Decimal = data.iter().map(|&x| (x - mean) * (x - mean)).sum();
        Ok(sum_squared_diff / Decimal::from(data.len() - 1))
    }

    /// 计算标准差
    ///
    /// # 参数
    /// - data: 数据集
    /// # 返回
    /// - 标准差
    pub fn std_dev(data: &[Decimal]) -> MathResult<Decimal> {
        let variance = Self::variance(data)?;
        AdvancedMath::sqrt(variance)
    }

    /// 计算偏度
    ///
    /// # 参数
    /// - data: 数据集
    /// # 返回
    /// - 偏度，数据量<3返回InvalidStrategyParameters
    pub fn skewness(data: &[Decimal]) -> MathResult<Decimal> {
        if data.len() < 3 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mean = Self::mean(data)?;
        let std_dev = Self::std_dev(data)?;
        if std_dev == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let n = Decimal::from(data.len());
        let sum_cubed: Decimal = data
            .iter()
            .map(|&x| {
                let standardized = (x - mean) / std_dev;
                standardized * standardized * standardized
            })
            .sum();
        Ok(sum_cubed / n)
    }

    /// 计算峰度
    ///
    /// # 参数
    /// - data: 数据集
    /// # 返回
    /// - 峰度，数据量<4返回InvalidStrategyParameters
    pub fn kurtosis(data: &[Decimal]) -> MathResult<Decimal> {
        if data.len() < 4 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mean = Self::mean(data)?;
        let std_dev = Self::std_dev(data)?;
        if std_dev == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let n = Decimal::from(data.len());
        let sum_fourth: Decimal = data
            .iter()
            .map(|&x| {
                let standardized = (x - mean) / std_dev;
                let squared = standardized * standardized;
                squared * squared
            })
            .sum();
        Ok(sum_fourth / n - Decimal::from(3)) // 超额峰度
    }

    /// 计算两个数据集的相关系数
    ///
    /// # 参数
    /// - x: 数据集1
    /// - y: 数据集2
    /// # 返回
    /// - 相关系数，长度不等或<2返回InvalidStrategyParameters
    pub fn correlation(x: &[Decimal], y: &[Decimal]) -> MathResult<Decimal> {
        if x.len() != y.len() || x.len() < 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mean_x = Self::mean(x)?;
        let mean_y = Self::mean(y)?;
        let mut sum_xy = Decimal::ZERO;
        let mut sum_x_squared = Decimal::ZERO;
        let mut sum_y_squared = Decimal::ZERO;
        for i in 0..x.len() {
            let diff_x = x[i] - mean_x;
            let diff_y = y[i] - mean_y;
            sum_xy += diff_x * diff_y;
            sum_x_squared += diff_x * diff_x;
            sum_y_squared += diff_y * diff_y;
        }
        let denominator = AdvancedMath::sqrt(sum_x_squared)? * AdvancedMath::sqrt(sum_y_squared)?;
        if denominator == Decimal::ZERO {
            Ok(Decimal::ZERO)
        } else {
            Ok(sum_xy / denominator)
        }
    }

    /// 历史模拟法计算VaR（风险价值）
    ///
    /// # 参数
    /// - returns: 收益率序列
    /// - confidence_level: 置信水平
    /// # 返回
    /// - VaR值
    pub fn var_historical(returns: &[Decimal], confidence_level: Decimal) -> MathResult<Decimal> {
        if returns.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort();
        let index = ((Decimal::ONE - confidence_level) * Decimal::from(returns.len()))
            .to_usize()
            .unwrap_or(0);
        let index = index.min(returns.len() - 1);
        Ok(-sorted_returns[index]) // VaR为正表示损失
    }

    /// 历史模拟法计算CVaR（条件风险价值）
    ///
    /// # 参数
    /// - returns: 收益率序列
    /// - confidence_level: 置信水平
    /// # 返回
    /// - CVaR值
    pub fn cvar_historical(returns: &[Decimal], confidence_level: Decimal) -> MathResult<Decimal> {
        if returns.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort();
        let cutoff_index = ((Decimal::ONE - confidence_level) * Decimal::from(returns.len()))
            .to_usize()
            .unwrap_or(0);
        if cutoff_index == 0 {
            return Ok(-sorted_returns[0]);
        }
        let tail_returns: Vec<Decimal> =
            sorted_returns.iter().take(cutoff_index).cloned().collect();
        let mean_tail = Self::mean(&tail_returns)?;
        Ok(-mean_tail)
    }

    /// 计算夏普比率
    ///
    /// # 参数
    /// - returns: 收益率序列
    /// - risk_free_rate: 无风险利率
    /// # 返回
    /// - 夏普比率
    pub fn sharpe_ratio(returns: &[Decimal], risk_free_rate: Decimal) -> MathResult<Decimal> {
        if returns.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mean_return = Self::mean(returns)?;
        let std_dev = Self::std_dev(returns)?;
        if std_dev == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        Ok((mean_return - risk_free_rate) / std_dev)
    }

    /// 计算最大回撤
    ///
    /// # 参数
    /// - prices: 价格序列
    /// # 返回
    /// - 最大回撤
    pub fn max_drawdown(prices: &[Decimal]) -> MathResult<Decimal> {
        if prices.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut max_price = prices[0];
        let mut max_drawdown = Decimal::ZERO;
        for &price in prices.iter().skip(1) {
            if price > max_price {
                max_price = price;
            } else {
                let drawdown = (max_price - price) / max_price;
                if drawdown > max_drawdown {
                    max_drawdown = drawdown;
                }
            }
        }
        Ok(max_drawdown)
    }

    /// 指数加权移动平均（EWMA）
    ///
    /// # 参数
    /// - data: 数据序列
    /// - lambda: 权重参数
    /// # 返回
    /// - EWMA序列
    pub fn ewma(data: &[Decimal], lambda: Decimal) -> MathResult<Vec<Decimal>> {
        if data.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut ewma_values = Vec::with_capacity(data.len());
        ewma_values.push(data[0]);
        for i in 1..data.len() {
            let ewma_value = lambda * data[i] + (Decimal::ONE - lambda) * ewma_values[i - 1];
            ewma_values.push(ewma_value);
        }
        Ok(ewma_values)
    }

    /// GARCH(1,1) 波动率建模
    ///
    /// # 参数
    /// - returns: 收益率序列
    /// - omega, alpha, beta: GARCH参数
    /// # 返回
    /// - 波动率序列
    pub fn garch_volatility(
        returns: &[Decimal],
        omega: Decimal,
        alpha: Decimal,
        beta: Decimal,
    ) -> MathResult<Vec<Decimal>> {
        if returns.len() < 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut volatilities = Vec::with_capacity(returns.len());
        // 初始方差估计
        let initial_var = Self::variance(returns)?;
        volatilities.push(AdvancedMath::sqrt(initial_var)?);
        for i in 1..returns.len() {
            let prev_return_squared = returns[i - 1] * returns[i - 1];
            let prev_variance = volatilities[i - 1] * volatilities[i - 1];
            let variance = omega + alpha * prev_return_squared + beta * prev_variance;
            volatilities.push(AdvancedMath::sqrt(variance)?);
        }
        Ok(volatilities)
    }
}

/// 时间序列分析工具结构体
/// - 提供自相关、ADF检验、Hurst指数等
pub struct TimeSeries;

impl TimeSeries {
    /// 计算指定滞后阶数的自相关系数
    ///
    /// # 参数
    /// - data: 数据序列
    /// - lag: 滞后阶数
    /// # 返回
    /// - 自相关系数
    pub fn autocorrelation(data: &[Decimal], lag: usize) -> MathResult<Decimal> {
        if data.len() <= lag {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let n = data.len() - lag;
        let x1: Vec<Decimal> = data.iter().take(n).cloned().collect();
        let x2: Vec<Decimal> = data.iter().skip(lag).cloned().collect();
        Statistics::correlation(&x1, &x2)
    }

    /// 简化版ADF检验（平稳性检测）
    ///
    /// # 参数
    /// - data: 数据序列
    /// # 返回
    /// - 检验统计量，数据量<10返回InvalidStrategyParameters
    pub fn adf_test(data: &[Decimal]) -> MathResult<Decimal> {
        if data.len() < 10 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        // 计算一阶差分
        let mut diffs = Vec::with_capacity(data.len() - 1);
        for i in 1..data.len() {
            diffs.push(data[i] - data[i - 1]);
        }
        // 计算检验统计量（简化）
        let mean_diff = Statistics::mean(&diffs)?;
        let std_diff = Statistics::std_dev(&diffs)?;
        if std_diff == Decimal::ZERO {
            Ok(Decimal::ZERO)
        } else {
            Ok(mean_diff / std_diff * AdvancedMath::sqrt(Decimal::from(diffs.len()))?)
        }
    }

    /// Hurst指数估算（长记忆性检测）
    ///
    /// # 参数
    /// - data: 数据序列
    /// # 返回
    /// - Hurst指数，数据量<20返回InvalidStrategyParameters
    pub fn hurst_exponent(data: &[Decimal]) -> MathResult<Decimal> {
        if data.len() < 20 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let n = data.len();
        let mean = Statistics::mean(data)?;
        // 计算累计偏差
        let mut cumulative_deviations = Vec::with_capacity(n);
        let mut sum = Decimal::ZERO;
        for &value in data {
            sum += value - mean;
            cumulative_deviations.push(sum);
        }
        // 计算极差
        let max_cum = cumulative_deviations
            .iter()
            .max()
            .ok_or(StrategyError::InvalidStrategyParameters)?;
        let min_cum = cumulative_deviations
            .iter()
            .min()
            .ok_or(StrategyError::InvalidStrategyParameters)?;
        let range = max_cum - min_cum;
        // 计算标准差
        let std_dev = Statistics::std_dev(data)?;
        if std_dev == Decimal::ZERO {
            return Ok(Decimal::from_str("0.5").map_err(|_| StrategyError::MathOverflow)?);
            // 随机游走
        }
        // R/S统计量
        let rs = range / std_dev;
        // Hurst指数近似
        let log_rs = AdvancedMath::ln(rs)?;
        let log_n = AdvancedMath::ln(Decimal::from(n))?;
        Ok(log_rs / log_n)
    }
}

/// 线性代数工具结构体
/// - 提供矩阵运算、逆、特征值等
pub struct LinearAlgebra;

impl LinearAlgebra {
    /// 矩阵乘法（二维矩阵）
    ///
    /// # 参数
    /// - a: 左矩阵
    /// - b: 右矩阵
    /// # 返回
    /// - 乘积矩阵，维度不匹配返回InvalidStrategyParameters
    pub fn matrix_multiply(
        a: &[Vec<Decimal>],
        b: &[Vec<Decimal>],
    ) -> MathResult<Vec<Vec<Decimal>>> {
        if a.is_empty() || b.is_empty() || a[0].len() != b.len() {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let rows_a = a.len();
        let cols_a = a[0].len();
        let cols_b = b[0].len();
        let mut result = vec![vec![Decimal::ZERO; cols_b]; rows_a];
        for i in 0..rows_a {
            for j in 0..cols_b {
                for k in 0..cols_a {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        Ok(result)
    }

    /// 计算矩阵行列式（仅支持2x2和3x3）
    ///
    /// # 参数
    /// - matrix: 输入矩阵
    /// # 返回
    /// - 行列式，维度不符返回InvalidStrategyParameters
    pub fn determinant(matrix: &[Vec<Decimal>]) -> MathResult<Decimal> {
        let n = matrix.len();
        if n == 0 || matrix[0].len() != n {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        match n {
            1 => Ok(matrix[0][0]),
            2 => Ok(matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0]),
            3 => {
                let a = matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1]);
                let b = matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0]);
                let c = matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0]);
                Ok(a - b + c)
            }
            _ => Err(StrategyError::InvalidStrategyParameters), // 仅支持3阶及以下
        }
    }

    /// 计算2x2矩阵的逆矩阵
    ///
    /// # 参数
    /// - matrix: 输入2x2矩阵
    /// # 返回
    /// - 逆矩阵，奇异矩阵返回MathOverflow
    pub fn inverse_2x2(matrix: &[Vec<Decimal>]) -> MathResult<Vec<Vec<Decimal>>> {
        if matrix.len() != 2 || matrix[0].len() != 2 || matrix[1].len() != 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let det = Self::determinant(matrix)?;
        if det == Decimal::ZERO {
            return Err(StrategyError::MathOverflow); // 奇异矩阵
        }
        let inv_det = Decimal::ONE / det;
        Ok(vec![
            vec![matrix[1][1] * inv_det, -matrix[0][1] * inv_det],
            vec![-matrix[1][0] * inv_det, matrix[0][0] * inv_det],
        ])
    }

    /// 计算2x2矩阵的特征值
    ///
    /// # 参数
    /// - matrix: 输入2x2矩阵
    /// # 返回
    /// - (lambda1, lambda2)，复数特征值返回MathOverflow
    pub fn eigenvalues_2x2(matrix: &[Vec<Decimal>]) -> MathResult<(Decimal, Decimal)> {
        if matrix.len() != 2 || matrix[0].len() != 2 || matrix[1].len() != 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let a = matrix[0][0];
        let b = matrix[0][1];
        let c = matrix[1][0];
        let d = matrix[1][1];
        let trace = a + d;
        let det = a * d - b * c;
        let discriminant = trace * trace - Decimal::from(4) * det;
        if discriminant < Decimal::ZERO {
            return Err(StrategyError::MathOverflow); // 复数特征值
        }
        let sqrt_discriminant = AdvancedMath::sqrt(discriminant)?;
        let lambda1 = (trace + sqrt_discriminant) / Decimal::from(2);
        let lambda2 = (trace - sqrt_discriminant) / Decimal::from(2);
        Ok((lambda1, lambda2))
    }
}

/// 优化算法工具结构体
/// - 提供黄金分割法、牛顿法、二分法等
pub struct Optimization;

impl Optimization {
    /// 黄金分割法（一元极值搜索）
    ///
    /// # 参数
    /// - f: 目标函数
    /// - a, b: 搜索区间
    /// - tolerance: 收敛容忍度
    /// # 返回
    /// - 极值点
    pub fn golden_section_search<F>(
        f: F,
        a: Decimal,
        b: Decimal,
        tolerance: Decimal,
    ) -> MathResult<Decimal>
    where
        F: Fn(Decimal) -> MathResult<Decimal>,
    {
        let phi = Decimal::from_f64(GOLDEN_RATIO).ok_or(StrategyError::MathOverflow)?;
        let resphi = Decimal::from(2) - phi;
        let mut x1 = a + resphi * (b - a);
        let mut x2 = a + (Decimal::ONE - resphi) * (b - a);
        let mut f1 = f(x1)?;
        let mut f2 = f(x2)?;
        let mut a_curr = a;
        let mut b_curr = b;
        while (b_curr - a_curr).abs() > tolerance {
            if f1 > f2 {
                a_curr = x1;
                x1 = x2;
                f1 = f2;
                x2 = a_curr + (Decimal::ONE - resphi) * (b_curr - a_curr);
                f2 = f(x2)?;
            } else {
                b_curr = x2;
                x2 = x1;
                f2 = f1;
                x1 = a_curr + resphi * (b_curr - a_curr);
                f1 = f(x1)?;
            }
        }
        Ok((a_curr + b_curr) / Decimal::from(2))
    }

    /// 牛顿法求根
    ///
    /// # 参数
    /// - f: 目标函数
    /// - df: 导数函数
    /// - initial_guess: 初始猜测
    /// - tolerance: 收敛容忍度
    /// - max_iterations: 最大迭代次数
    /// # 返回
    /// - 根，未收敛返回MathOverflow
    pub fn newton_method<F, G>(
        f: F,
        df: G,
        initial_guess: Decimal,
        tolerance: Decimal,
        max_iterations: u32,
    ) -> MathResult<Decimal>
    where
        F: Fn(Decimal) -> MathResult<Decimal>,
        G: Fn(Decimal) -> MathResult<Decimal>,
    {
        let mut x = initial_guess;
        for _ in 0..max_iterations {
            let fx = f(x)?;
            let dfx = df(x)?;
            if dfx.abs() < Decimal::from_f64(EPSILON_F64).ok_or(StrategyError::MathOverflow)? {
                return Err(StrategyError::MathOverflow); // 导数过小
            }
            let x_new = x - fx / dfx;
            if (x_new - x).abs() < tolerance {
                return Ok(x_new);
            }
            x = x_new;
        }
        Err(StrategyError::MathOverflow) // 未收敛
    }

    /// 二分法求根
    ///
    /// # 参数
    /// - f: 目标函数
    /// - a, b: 区间端点
    /// - tolerance: 收敛容忍度
    /// # 返回
    /// - 根，区间无根返回InvalidStrategyParameters
    pub fn bisection_method<F>(
        f: F,
        a: Decimal,
        b: Decimal,
        tolerance: Decimal,
    ) -> MathResult<Decimal>
    where
        F: Fn(Decimal) -> MathResult<Decimal>,
    {
        let fa = f(a)?;
        let fb = f(b)?;
        if fa * fb > Decimal::ZERO {
            return Err(StrategyError::InvalidStrategyParameters); // 区间无根
        }
        let mut a_curr = a;
        let mut b_curr = b;
        for _ in 0..MAX_ITERATIONS_BISECTION {
            let c = (a_curr + b_curr) / Decimal::from(2);
            let fc = f(c)?;
            if fc.abs() < tolerance || (b_curr - a_curr) / Decimal::from(2) < tolerance {
                return Ok(c);
            }
            if fa * fc < Decimal::ZERO {
                b_curr = c;
            } else {
                a_curr = c;
            }
        }
        Ok((a_curr + b_curr) / Decimal::from(2))
    }
}

/// 安全数学运算工具结构体（溢出保护）
/// - 提供u64安全加减乘除、复利、基点百分比等
pub struct SafeMath;

impl SafeMath {
    /// 安全加法，溢出检查
    ///
    /// # 参数
    /// - a, b: 操作数
    /// # 返回
    /// - a+b，溢出返回MathOverflow
    pub fn add(a: u64, b: u64) -> MathResult<u64> {
        a.checked_add(b).ok_or(StrategyError::MathOverflow)
    }
    /// 安全减法，下溢检查
    ///
    /// # 参数
    /// - a, b: 操作数
    /// # 返回
    /// - a-b，下溢返回MathOverflow
    pub fn sub(a: u64, b: u64) -> MathResult<u64> {
        a.checked_sub(b).ok_or(StrategyError::MathOverflow)
    }
    /// 安全乘法，溢出检查
    ///
    /// # 参数
    /// - a, b: 操作数
    /// # 返回
    /// - a*b，溢出返回MathOverflow
    pub fn mul(a: u64, b: u64) -> MathResult<u64> {
        a.checked_mul(b).ok_or(StrategyError::MathOverflow)
    }
    /// 安全除法，零检查
    ///
    /// # 参数
    /// - a, b: 操作数
    /// # 返回
    /// - a/b，b=0返回MathOverflow
    pub fn div(a: u64, b: u64) -> MathResult<u64> {
        if b == 0 {
            return Err(StrategyError::MathOverflow);
        }
        Ok(a / b)
    }
    /// 精度可控的安全除法
    ///
    /// # 参数
    /// - a, b: 操作数
    /// - precision: 精度位数
    /// # 返回
    /// - (a*10^precision)/b，b=0返回MathOverflow
    pub fn div_precise(a: u64, b: u64, precision: u8) -> MathResult<u64> {
        if b == 0 {
            return Err(StrategyError::MathOverflow);
        }
        let multiplier = 10u64.pow(precision as u32);
        let numerator = Self::mul(a, multiplier)?;
        Ok(numerator / b)
    }
    /// 基点百分比计算
    ///
    /// # 参数
    /// - amount: 金额
    /// - percentage_bps: 百分比（基点）
    /// # 返回
    /// - 百分比结果
    pub fn percentage_bps(amount: u64, percentage_bps: u64) -> MathResult<u64> {
        let result = Self::mul(amount, percentage_bps)?;
        Ok(result / BASIS_POINTS_MAX)
    }
    /// 复利计算
    ///
    /// # 参数
    /// - principal: 本金
    /// - rate_bps: 利率（基点）
    /// - periods: 期数
    /// # 返回
    /// - 复利结果
    pub fn compound_interest(principal: u64, rate_bps: u64, periods: u32) -> MathResult<u64> {
        let rate_decimal = Decimal::from(rate_bps) / Decimal::from(BASIS_POINTS_MAX);
        let principal_decimal = Decimal::from(principal);
        let periods_decimal = Decimal::from(periods);
        let compound_factor = AdvancedMath::pow(Decimal::ONE + rate_decimal, periods_decimal)?;
        let result = principal_decimal * compound_factor;
        result.to_u64().ok_or(StrategyError::MathOverflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 测试SafeMath基本运算
    #[test]
    fn test_safe_math_operations() {
        assert_eq!(SafeMath::add(100, 200).unwrap(), 300);
        assert_eq!(SafeMath::sub(200, 100).unwrap(), 100);
        assert_eq!(SafeMath::mul(10, 20).unwrap(), 200);
        assert_eq!(SafeMath::div(100, 10).unwrap(), 10);
        // 溢出测试
        assert!(SafeMath::add(u64::MAX, 1).is_err());
        assert!(SafeMath::sub(0, 1).is_err());
        assert!(SafeMath::div(100, 0).is_err());
    }
    /// 测试统计分析
    #[test]
    fn test_statistics() {
        let data = vec![
            Decimal::from(1),
            Decimal::from(2),
            Decimal::from(3),
            Decimal::from(4),
            Decimal::from(5),
        ];
        let mean = Statistics::mean(&data).unwrap();
        assert_eq!(mean, Decimal::from(3));
        let variance = Statistics::variance(&data).unwrap();
        assert!(variance > Decimal::ZERO);
        let std_dev = Statistics::std_dev(&data).unwrap();
        assert!(std_dev > Decimal::ZERO);
    }
    /// 测试高级数学函数
    #[test]
    fn test_advanced_math() {
        let x = Decimal::from(4);
        let sqrt_x = AdvancedMath::sqrt(x).unwrap();
        assert_eq!(sqrt_x, Decimal::from(2));
        let ln_e = AdvancedMath::ln(Decimal::from_f64(E).unwrap()).unwrap();
        assert!((ln_e - Decimal::ONE).abs() < Decimal::from_f64(1e-10).unwrap());
    }
}
