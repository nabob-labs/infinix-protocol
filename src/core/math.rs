/*!
 * Advanced Mathematical Utilities
 *
 * Production-ready mathematical functions and utilities for:
 * - Statistical calculations and distributions
 * - Numerical optimization algorithms
 * - Financial mathematics and risk calculations
 * - Linear algebra operations
 * - Time series analysis
 * - Machine learning mathematical foundations
 */

use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// Result type for mathematical operations
pub type MathResult<T> = Result<T>;

/// Advanced mathematical utilities
pub struct AdvancedMath;

impl AdvancedMath {
    /// Calculate natural logarithm with high precision
    pub fn ln(x: Decimal) -> MathResult<Decimal> {
        if x <= Decimal::ZERO {
            return Err(StrategyError::MathOverflow);
        }

        // Use Taylor series for ln(1+x) when x is close to 0
        let x_f64 = x.to_f64().ok_or(StrategyError::MathOverflow)?;
        let result = x_f64.ln();

        if result.is_finite() {
            Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
        } else {
            Err(StrategyError::MathOverflow)
        }
    }

    /// Calculate exponential function with overflow protection
    pub fn exp(x: Decimal) -> MathResult<Decimal> {
        let x_f64 = x.to_f64().ok_or(StrategyError::MathOverflow)?;

        // Prevent overflow
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

    /// Calculate square root using Newton's method
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

    /// Calculate power function x^y
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

    /// Calculate normal distribution cumulative density function
    pub fn normal_cdf(x: Decimal) -> MathResult<Decimal> {
        let x_f64 = x.to_f64().ok_or(StrategyError::MathOverflow)?;

        // Abramowitz and Stegun approximation
        let t = 1.0 / (1.0 + 0.2316419 * x_f64.abs());
        let d = 0.3989423 * (-x_f64 * x_f64 / 2.0).exp();
        let prob = d
            * t
            * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));

        let result = if x_f64 >= 0.0 { 1.0 - prob } else { prob };

        Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
    }

    /// Calculate inverse normal distribution (quantile function)
    pub fn normal_inv(p: Decimal) -> MathResult<Decimal> {
        let p_f64 = p.to_f64().ok_or(StrategyError::MathOverflow)?;

        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Err(StrategyError::InvalidStrategyParameters);
        }

        // Simplified approximation for inverse normal distribution
        // Using Beasley-Springer-Moro algorithm coefficients
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
            // Rational approximation for lower region
            let q = (-2.0 * p_f64.ln()).sqrt();
            let num = ((((c[5] * q + c[4]) * q + c[3]) * q + c[2]) * q + c[1]) * q + c[0];
            let den = (((d[3] * q + d[2]) * q + d[1]) * q + d[0]) * q + 1.0;
            num / den
        } else if p_f64 <= p_high {
            // Rational approximation for central region
            let q = p_f64 - 0.5;
            let r = q * q;
            let num = (((((a[5] * r + a[4]) * r + a[3]) * r + a[2]) * r + a[1]) * r + a[0]) * q;
            let den = ((((b[4] * r + b[3]) * r + b[2]) * r + b[1]) * r + b[0]) * r + 1.0;
            num / den
        } else {
            // Rational approximation for upper region
            let q = (-2.0 * (1.0 - p_f64).ln()).sqrt();
            let num = ((((c[5] * q + c[4]) * q + c[3]) * q + c[2]) * q + c[1]) * q + c[0];
            let den = (((d[3] * q + d[2]) * q + d[1]) * q + d[0]) * q + 1.0;
            -(num / den)
        };

        Decimal::from_f64(result).ok_or(StrategyError::MathOverflow)
    }

    /// Calculate Black-Scholes option price
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

    /// Calculate implied volatility using Newton-Raphson method
    pub fn implied_volatility(
        market_price: Decimal,
        spot: Decimal,
        strike: Decimal,
        time_to_expiry: Decimal,
        risk_free_rate: Decimal,
    ) -> MathResult<Decimal> {
        let mut vol = Decimal::from_str("0.2").map_err(|_| StrategyError::MathOverflow)?; // Initial guess: 20%

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

            // Ensure volatility stays positive
            if vol <= Decimal::ZERO {
                vol = Decimal::from_str("0.001").map_err(|_| StrategyError::MathOverflow)?;
            }
        }

        Ok(vol)
    }

    /// Calculate Black-Scholes vega (sensitivity to volatility)
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

/// Statistical utilities
pub struct Statistics;

impl Statistics {
    /// Calculate mean of a dataset
    pub fn mean(data: &[Decimal]) -> MathResult<Decimal> {
        if data.is_empty() {
            return Err(StrategyError::InvalidStrategyParameters);
        }

        let sum: Decimal = data.iter().sum();
        Ok(sum / Decimal::from(data.len()))
    }

    /// Calculate variance of a dataset
    pub fn variance(data: &[Decimal]) -> MathResult<Decimal> {
        if data.len() < 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }

        let mean = Self::mean(data)?;
        let sum_squared_diff: Decimal = data.iter().map(|&x| (x - mean) * (x - mean)).sum();

        Ok(sum_squared_diff / Decimal::from(data.len() - 1))
    }

    /// Calculate standard deviation
    pub fn std_dev(data: &[Decimal]) -> MathResult<Decimal> {
        let variance = Self::variance(data)?;
        AdvancedMath::sqrt(variance)
    }

    /// Calculate skewness
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

    /// Calculate kurtosis
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

        Ok(sum_fourth / n - Decimal::from(3)) // Excess kurtosis
    }

    /// Calculate correlation coefficient between two datasets
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

    /// Calculate Value at Risk using historical simulation
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

        Ok(-sorted_returns[index]) // VaR is positive for losses
    }

    /// Calculate Conditional Value at Risk (Expected Shortfall)
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

    /// Calculate Sharpe ratio
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

    /// Calculate maximum drawdown
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

    /// Calculate exponentially weighted moving average
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

    /// Calculate GARCH(1,1) volatility
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

        // Initial volatility estimate
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

/// Time series analysis utilities
pub struct TimeSeries;

impl TimeSeries {
    /// Calculate autocorrelation at given lag
    pub fn autocorrelation(data: &[Decimal], lag: usize) -> MathResult<Decimal> {
        if data.len() <= lag {
            return Err(StrategyError::InvalidStrategyParameters);
        }

        let n = data.len() - lag;
        let x1: Vec<Decimal> = data.iter().take(n).cloned().collect();
        let x2: Vec<Decimal> = data.iter().skip(lag).cloned().collect();

        Statistics::correlation(&x1, &x2)
    }

    /// Perform Augmented Dickey-Fuller test for stationarity
    pub fn adf_test(data: &[Decimal]) -> MathResult<Decimal> {
        if data.len() < 10 {
            return Err(StrategyError::InvalidStrategyParameters);
        }

        // Simplified ADF test - calculate first difference
        let mut diffs = Vec::with_capacity(data.len() - 1);
        for i in 1..data.len() {
            diffs.push(data[i] - data[i - 1]);
        }

        // Calculate test statistic (simplified)
        let mean_diff = Statistics::mean(&diffs)?;
        let std_diff = Statistics::std_dev(&diffs)?;

        if std_diff == Decimal::ZERO {
            Ok(Decimal::ZERO)
        } else {
            Ok(mean_diff / std_diff * AdvancedMath::sqrt(Decimal::from(diffs.len()))?)
        }
    }

    /// Calculate Hurst exponent for long-term memory detection
    pub fn hurst_exponent(data: &[Decimal]) -> MathResult<Decimal> {
        if data.len() < 20 {
            return Err(StrategyError::InvalidStrategyParameters);
        }

        let n = data.len();
        let mean = Statistics::mean(data)?;

        // Calculate cumulative deviations
        let mut cumulative_deviations = Vec::with_capacity(n);
        let mut sum = Decimal::ZERO;

        for &value in data {
            sum += value - mean;
            cumulative_deviations.push(sum);
        }

        // Calculate range
        let max_cum = cumulative_deviations
            .iter()
            .max()
            .ok_or(StrategyError::InvalidStrategyParameters)?;
        let min_cum = cumulative_deviations
            .iter()
            .min()
            .ok_or(StrategyError::InvalidStrategyParameters)?;
        let range = max_cum - min_cum;

        // Calculate standard deviation
        let std_dev = Statistics::std_dev(data)?;

        if std_dev == Decimal::ZERO {
            return Ok(Decimal::from_str("0.5").map_err(|_| StrategyError::MathOverflow)?);
            // Random walk
        }

        // R/S statistic
        let rs = range / std_dev;

        // Hurst exponent approximation
        let log_rs = AdvancedMath::ln(rs)?;
        let log_n = AdvancedMath::ln(Decimal::from(n))?;

        Ok(log_rs / log_n)
    }
}

/// Linear algebra utilities
pub struct LinearAlgebra;

impl LinearAlgebra {
    /// Matrix multiplication for 2D matrices represented as Vec<Vec<Decimal>>
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

    /// Calculate matrix determinant (2x2 and 3x3 only)
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
            _ => Err(StrategyError::InvalidStrategyParameters), // Only support up to 3x3
        }
    }

    /// Calculate matrix inverse (2x2 only for simplicity)
    pub fn inverse_2x2(matrix: &[Vec<Decimal>]) -> MathResult<Vec<Vec<Decimal>>> {
        if matrix.len() != 2 || matrix[0].len() != 2 || matrix[1].len() != 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }

        let det = Self::determinant(matrix)?;

        if det == Decimal::ZERO {
            return Err(StrategyError::MathOverflow); // Singular matrix
        }

        let inv_det = Decimal::ONE / det;

        Ok(vec![
            vec![matrix[1][1] * inv_det, -matrix[0][1] * inv_det],
            vec![-matrix[1][0] * inv_det, matrix[0][0] * inv_det],
        ])
    }

    /// Calculate eigenvalues for 2x2 matrix
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
            return Err(StrategyError::MathOverflow); // Complex eigenvalues
        }

        let sqrt_discriminant = AdvancedMath::sqrt(discriminant)?;
        let lambda1 = (trace + sqrt_discriminant) / Decimal::from(2);
        let lambda2 = (trace - sqrt_discriminant) / Decimal::from(2);

        Ok((lambda1, lambda2))
    }
}

/// Optimization utilities
pub struct Optimization;

impl Optimization {
    /// Golden section search for univariate optimization
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

    /// Newton's method for root finding
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
                return Err(StrategyError::MathOverflow); // Derivative too small
            }

            let x_new = x - fx / dfx;

            if (x_new - x).abs() < tolerance {
                return Ok(x_new);
            }

            x = x_new;
        }

        Err(StrategyError::MathOverflow) // Failed to converge
    }

    /// Bisection method for root finding
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
            return Err(StrategyError::InvalidStrategyParameters); // No root in interval
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

/// Safe mathematical operations with overflow protection
pub struct SafeMath;

impl SafeMath {
    /// Safe addition with overflow check
    pub fn add(a: u64, b: u64) -> MathResult<u64> {
        a.checked_add(b).ok_or(StrategyError::MathOverflow)
    }

    /// Safe subtraction with underflow check
    pub fn sub(a: u64, b: u64) -> MathResult<u64> {
        a.checked_sub(b).ok_or(StrategyError::MathOverflow)
    }

    /// Safe multiplication with overflow check
    pub fn mul(a: u64, b: u64) -> MathResult<u64> {
        a.checked_mul(b).ok_or(StrategyError::MathOverflow)
    }

    /// Safe division with zero check
    pub fn div(a: u64, b: u64) -> MathResult<u64> {
        if b == 0 {
            return Err(StrategyError::MathOverflow);
        }
        Ok(a / b)
    }

    /// Safe division with precision
    pub fn div_precise(a: u64, b: u64, precision: u8) -> MathResult<u64> {
        if b == 0 {
            return Err(StrategyError::MathOverflow);
        }

        let multiplier = 10u64.pow(precision as u32);
        let numerator = Self::mul(a, multiplier)?;
        Ok(numerator / b)
    }

    /// Calculate percentage with basis points precision
    pub fn percentage_bps(amount: u64, percentage_bps: u64) -> MathResult<u64> {
        let result = Self::mul(amount, percentage_bps)?;
        Ok(result / BASIS_POINTS_MAX)
    }

    /// Calculate compound interest
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

    #[test]
    fn test_safe_math_operations() {
        assert_eq!(SafeMath::add(100, 200).unwrap(), 300);
        assert_eq!(SafeMath::sub(200, 100).unwrap(), 100);
        assert_eq!(SafeMath::mul(10, 20).unwrap(), 200);
        assert_eq!(SafeMath::div(100, 10).unwrap(), 10);

        // Test overflow
        assert!(SafeMath::add(u64::MAX, 1).is_err());
        assert!(SafeMath::sub(0, 1).is_err());
        assert!(SafeMath::div(100, 0).is_err());
    }

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

    #[test]
    fn test_advanced_math() {
        let x = Decimal::from(4);
        let sqrt_x = AdvancedMath::sqrt(x).unwrap();
        assert_eq!(sqrt_x, Decimal::from(2));

        let ln_e = AdvancedMath::ln(Decimal::from_f64(E).unwrap()).unwrap();
        assert!((ln_e - Decimal::ONE).abs() < Decimal::from_f64(1e-10).unwrap());
    }
}
