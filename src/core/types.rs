/*!
 * Core Types Module - Optimized for Anchor 0.31.1
 *
 * 本模块定义了系统中广泛使用的基础类型，具备：
 * - 全面的校验与错误处理
 * - 类型安全与边界检查
 * - 性能优化的数据结构
 * - 针对 Solana 的特殊优化
 * - 清晰的文档与用法示例
 */

use anchor_lang::prelude::*;

/// 统一交易参数结构体
/// - 适用于所有资产/篮子/指数代币的单笔交易指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TradeParams {
    /// 交易类型（如 swap、mint、burn、rebalance 等）
    pub trade_type: String,
    /// 源资产/篮子/指数代币 mint
    pub from_token: Pubkey,
    /// 目标资产/篮子/指数代币 mint
    pub to_token: Pubkey,
    /// 输入数量
    pub amount_in: u64,
    /// 最小输出数量
    pub min_amount_out: u64,
    /// DEX 名称
    pub dex_name: String,
    /// 算法参数（可选）
    pub algo_params: Option<AlgoParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
    /// 预言机参数（可选）
    pub oracle_params: Option<OracleParams>,
}

/// 统一批量交易参数结构体
/// - 适用于批量操作
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchTradeParams {
    /// 批量交易明细
    pub trades: Vec<TradeParams>,
}

/// 算法参数结构体
/// - 适用于所有算法融合指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AlgoParams {
    /// 算法名称
    pub algo_name: String,
    /// 算法参数序列化数据
    pub params: Vec<u8>,
}

/// DEX 参数结构体
/// - 适用于所有 DEX/AMM 指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DexParams {
    /// DEX 名称
    pub dex_name: String,
    /// DEX 参数序列化数据
    pub params: Vec<u8>,
}

/// 预言机参数结构体
/// - 适用于所有 Oracle 指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OracleParams {
    /// 预言机名称
    pub oracle_name: String,
    /// 预言机参数序列化数据
    pub params: Vec<u8>,
}

/// 策略参数结构体
/// - 适用于所有策略融合指令
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StrategyParams {
    /// 策略名称
    pub strategy_name: String,
    /// 策略参数序列化数据
    pub params: Vec<u8>,
}

// ============================================================================
// RISK MANAGEMENT TYPES
// ============================================================================

/// 投资组合风险指标结构体
/// - 所有数值均以基点（10000=100%）存储，保证精度
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct RiskMetrics {
    /// 95%置信区间VaR（基点）
    pub var_95: u64,
    /// 99%置信区间VaR（基点）
    pub var_99: u64,
    /// 最大回撤（基点）
    pub max_drawdown: u64,
    /// 投资组合波动率（基点）
    pub volatility: u64,
    /// Sharpe比率*10000
    pub sharpe_ratio: i64,
    /// Beta*10000（1.0=10000）
    pub beta: i64,
    /// 当前周期VaR（基点）
    pub var_bps: u64,
    /// 集中度风险（基点）
    pub concentration_risk: u64,
    /// 总体风险评分（0-10000，越高越风险大）
    pub overall_risk_score: u32,
    /// 历史最大回撤（基点）
    pub max_drawdown_bps: u64,
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            var_95: 0,
            var_99: 0,
            max_drawdown: 0,
            volatility: 0,
            sharpe_ratio: 0,
            beta: 10_000, // Beta of 1.0
            var_bps: 0,
            concentration_risk: 0,
            overall_risk_score: 0,
            max_drawdown_bps: 0,
        }
    }
}

impl RiskMetrics {
    /// 构造带校验的风险指标
    /// - 参数均为基点，需满足合理边界
    /// - 返回 Ok(Self) 或 Err(StrategyError)
    pub fn new(
        var_95: u64,
        var_99: u64,
        max_drawdown: u64,
        volatility: u64,
        sharpe_ratio: i64,
        beta: i64,
        var_bps: u64,
        concentration_risk: u64,
        overall_risk_score: u32,
        max_drawdown_bps: u64,
    ) -> Result<Self> {
        // 校验所有输入在合理范围
        require!(
            var_95 <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            var_99 <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_drawdown <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            volatility <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            var_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            concentration_risk <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            overall_risk_score <= 10_000,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_drawdown_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );

        // 校验逻辑关系
        require!(
            var_99 >= var_95,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            max_drawdown >= max_drawdown_bps,
            crate::error::StrategyError::InvalidStrategyParameters
        );

        Ok(Self {
            var_95,
            var_99,
            max_drawdown,
            volatility,
            sharpe_ratio,
            beta,
            var_bps,
            concentration_risk,
            overall_risk_score,
            max_drawdown_bps,
        })
    }

    /// 判断是否为高风险
    pub fn is_high_risk(&self) -> bool {
        self.overall_risk_score > 7_000 || // 70% 风险评分
        self.var_95 > 2_000 || // 20% VaR
        self.concentration_risk > 3_000 || // 30% 集中度
        self.max_drawdown > 1_500 // 15% 回撤
    }

    /// 判断风险指标是否在限额内
    pub fn is_within_limits(&self, limits: &RiskLimits) -> bool {
        self.var_95 <= limits.max_var_bps
            && self.concentration_risk <= limits.max_concentration_bps
            && self.max_drawdown <= limits.max_drawdown_bps
            && self.overall_risk_score <= limits.max_risk_score
    }

    /// 计算风险调整后收益
    pub fn risk_adjusted_return(&self, return_bps: i64) -> i64 {
        if self.volatility == 0 {
            return 0;
        }
        // Sharpe ratio calculation: (return - risk_free_rate) / volatility
        // Assuming 0% risk-free rate for simplicity
        (return_bps * 10_000) / self.volatility as i64
    }
}

// ============================================================================
// MARKET DATA TYPES
// ============================================================================

/// 市场数据结构体
/// - 用于交易决策的全面市场数据
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MarketData {
    /// 当前价格（最小单位）
    pub price: u64,
    /// 24小时交易量（最小单位）
    pub volume_24h: u64,
    /// 市值（最小单位）
    pub market_cap: u64,
    /// 可用流动性（最小单位）
    pub liquidity: u64,
    /// 数据采集时间戳
    pub timestamp: i64,
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            price: 0,
            volume_24h: 0,
            market_cap: 0,
            liquidity: 0,
            timestamp: 0,
        }
    }
}

impl MarketData {
    /// 构造带校验的市场数据
    /// - 价格必须大于0
    /// - 时间戳必须大于0
    /// - 返回 Ok(Self) 或 Err(StrategyError)
    pub fn new(
        price: u64,
        volume_24h: u64,
        market_cap: u64,
        liquidity: u64,
        timestamp: i64,
    ) -> Result<Self> {
        require!(price > 0, crate::error::StrategyError::InvalidMarketData);
        require!(
            timestamp > 0,
            crate::error::StrategyError::InvalidMarketData
        );

        Ok(Self {
            price,
            volume_24h,
            market_cap,
            liquidity,
            timestamp,
        })
    }

    /// 判断市场数据是否过时（超过5分钟）
    pub fn is_stale(&self, current_time: i64) -> bool {
        current_time - self.timestamp > 300 // 5 minutes
    }

    /// 从历史数据计算价格波动率
    pub fn calculate_volatility(&self, historical_prices: &[u64]) -> u64 {
        if historical_prices.len() < 2 {
            return 0;
        }

        let mut variance = 0u64;
        let mean = historical_prices.iter().sum::<u64>() / historical_prices.len() as u64;

        for &price in historical_prices {
            let diff = if price > mean {
                price - mean
            } else {
                mean - price
            };
            variance += diff * diff;
        }

        variance / historical_prices.len() as u64
    }

    /// 估算买卖价差
    pub fn estimate_spread(&self) -> u64 {
        // 基于流动性的简单价差估算
        if self.liquidity == 0 {
            return 100; // 1% 默认价差
        }

        let spread_bps = (self.volume_24h * 100) / self.liquidity;
        spread_bps.min(500) // 上限5%
    }
}

// ============================================================================
// TOKEN INFORMATION TYPES
// ============================================================================

/// 代币信息结构体
/// - 用于投资组合管理的全面代币信息
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct TokenInfo {
    /// 代币 mint 地址
    pub mint: Pubkey,
    /// 代币符号（如 "SOL", "USDC"）
    pub symbol: String,
    /// 代币小数位数（如 9 位 for SOL, 6 位 for USDC）
    pub decimals: u8,
    /// 当前价格（最小单位）
    pub price: u64,
    /// 是否激活用于交易
    pub is_active: bool,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            mint: Pubkey::default(),
            symbol: String::new(),
            decimals: 6,
            price: 0,
            is_active: true,
        }
    }
}

impl TokenInfo {
    /// 构造带校验的代币信息
    /// - 代币符号不能为空
    /// - 小数位数不能超过18
    /// - 价格必须大于0
    /// - 返回 Ok(Self) 或 Err(StrategyError)
    pub fn new(
        mint: Pubkey,
        symbol: String,
        decimals: u8,
        price: u64,
        is_active: bool,
    ) -> Result<Self> {
        require!(
            !symbol.is_empty(),
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            decimals <= 18,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(price > 0, crate::error::StrategyError::InvalidMarketData);

        Ok(Self {
            mint,
            symbol,
            decimals,
            price,
            is_active,
        })
    }

    /// 将最小单位数量转换为可读格式
    pub fn to_human_readable(&self, amount: u64) -> f64 {
        amount as f64 / (10_u64.pow(self.decimals as u32) as f64)
    }

    /// 将可读格式数量转换为最小单位
    pub fn from_human_readable(&self, amount: f64) -> u64 {
        (amount * 10_u64.pow(self.decimals as u32) as f64) as u64
    }

    /// 计算代币数量市值
    pub fn market_value(&self, amount: u64) -> u64 {
        amount * self.price / 10_u64.pow(self.decimals as u32)
    }
}

// ============================================================================
// 权重分配类型
// ============================================================================

/// 权重分配结构体
/// - 用于投资组合构建的权重分配
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct WeightAllocation {
    /// 代币 mint 地址
    pub token_mint: Pubkey,
    /// 权重（基点）
    pub weight_bps: u64,
}

impl Default for WeightAllocation {
    fn default() -> Self {
        Self {
            token_mint: Pubkey::default(),
            weight_bps: 0,
        }
    }
}

impl WeightAllocation {
    /// 构造带校验的权重分配
    /// - 权重必须小于等于最大权重（10000基点）
    /// - 返回 Ok(Self) 或 Err(StrategyError)
    pub fn new(token_mint: Pubkey, weight_bps: u64) -> Result<Self> {
        require!(
            weight_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        Ok(Self { token_mint, weight_bps })
    }

    /// 获取权重百分比（0.0 到 1.0）
    pub fn weight_percentage(&self) -> f64 {
        self.weight_bps as f64 / BASIS_POINTS_MAX as f64
    }

    /// 从百分比设置权重（0.0 到 1.0）
    pub fn set_weight_percentage(&mut self, percentage: f64) -> Result<()> {
        require!(
            percentage >= 0.0 && percentage <= 1.0,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        self.weight_bps = (percentage * BASIS_POINTS_MAX as f64) as u64;
        Ok(())
    }
}

// ============================================================================
// 风险限额类型
// ============================================================================

/// 风险限额结构体
/// - 用于投资组合管理的风险限制
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct RiskLimits {
    /// 最大VaR（基点）
    pub max_var_bps: u32,
    /// 最大集中度（基点）
    pub max_concentration_bps: u32,
    /// 最大回撤（基点）
    pub max_drawdown_bps: u32,
    /// 最大风险评分
    pub max_risk_score: u32,
    /// 启用熔断器
    pub enable_circuit_breakers: bool,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_var_bps: 2_000,
            max_concentration_bps: 3_000,
            max_drawdown_bps: 1_500,
            max_risk_score: 7_000,
            enable_circuit_breakers: true,
        }
    }
}

impl RiskLimits {
    /// 构造带校验的风险限额
    /// - 所有参数均为基点，需满足合理边界
    /// - 返回 Ok(Self) 或 Err(StrategyError)
    pub fn new(
        max_var_bps: u64,
        max_concentration_bps: u64,
        max_drawdown_bps: u64,
        max_risk_score: u32,
    ) -> Result<Self> {
        require!(max_var_bps <= BASIS_POINTS_MAX, crate::error::StrategyError::InvalidStrategyParameters);
        require!(max_concentration_bps <= BASIS_POINTS_MAX, crate::error::StrategyError::InvalidStrategyParameters);
        require!(max_drawdown_bps <= BASIS_POINTS_MAX, crate::error::StrategyError::InvalidStrategyParameters);
        require!(max_risk_score <= 10_000, crate::error::StrategyError::InvalidStrategyParameters);
        Ok(Self {
            max_var_bps: max_var_bps as u32,
            max_concentration_bps: max_concentration_bps as u32,
            max_drawdown_bps: max_drawdown_bps as u32,
            max_risk_score,
            enable_circuit_breakers: true,
        })
    }

    /// 判断风险指标是否违反限额
    pub fn is_violated(&self, metrics: &RiskMetrics) -> bool {
        metrics.var_95 > self.max_var_bps as u64
            || metrics.concentration_risk > self.max_concentration_bps as u64
            || metrics.max_drawdown > self.max_drawdown_bps as u64
            || metrics.overall_risk_score > self.max_risk_score
    }
}

// ============================================================================
// 可验证类型特征实现
// ============================================================================

/// 可验证类型特征
/// - 约定所有实现该 trait 的类型都必须实现 validate 方法
/// - 用于统一校验各类结构体的参数合法性，便于 Anchor 指令和业务逻辑调用
pub trait Validatable {
    /// 验证类型并返回 Result
    fn validate(&self) -> Result<()>;
}

/// RiskMetrics 的校验实现
impl Validatable for RiskMetrics {
    fn validate(&self) -> Result<()> {
        require!(
            self.var_95 <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.var_99 <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.max_drawdown <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.volatility <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.var_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.concentration_risk <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.overall_risk_score <= 10_000,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.max_drawdown_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        Ok(())
    }
}

/// MarketData 的校验实现
impl Validatable for MarketData {
    fn validate(&self) -> Result<()> {
        require!(
            self.price > 0,
            crate::error::StrategyError::InvalidMarketData
        );
        require!(
            self.timestamp > 0,
            crate::error::StrategyError::InvalidMarketData
        );
        Ok(())
    }
}

/// TokenInfo 的校验实现
impl Validatable for TokenInfo {
    fn validate(&self) -> Result<()> {
        require!(
            !self.symbol.is_empty(),
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.decimals <= 18,
            crate::error::StrategyError::InvalidStrategyParameters
        );
        require!(
            self.price > 0,
            crate::error::StrategyError::InvalidMarketData
        );
        Ok(())
    }
}

/// WeightAllocation 的校验实现
impl Validatable for WeightAllocation {
    fn validate(&self) -> Result<()> {
        require!(
            self.weight_bps <= BASIS_POINTS_MAX,
            crate::error::StrategyError::InvalidWeightSum
        );
        Ok(())
    }
}
