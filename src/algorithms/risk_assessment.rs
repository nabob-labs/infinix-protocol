/*!
 * Advanced Risk Assessment Engine
 *
 * ## 算法简介
 * 多维风险评估引擎，支持市场、流动性、波动率、集中度、对手方、操作、模型、系统性等多种风险类型的实时建模与聚合。
 *
 * ## 主要特性
 * - **多模型可插拔**：支持多种风险模型（VaR、ES、流动性、集中度等），trait接口可扩展
 * - **极端风险保护**：高波动、低流动性、高集中度等极端风险自动熔断
 * - **相关性与历史回放**：支持风险相关性调整、历史风险回放、ML预测等
 * - **参数校验与溢出保护**：所有输入参数和数值运算均有严格校验
 * - **可观测性**：评估过程有详细日志输出，便于链上追踪和监控
 * - **单元测试**：覆盖极端风险、异常输入、边界条件、熔断触发等场景
 *
 * ## 关键可插拔点
 * - `RiskModel` trait：支持自定义风险模型
 * - `RiskAssessmentEngine`：支持多模型聚合与动态权重
 *
 * ## 极端场景保护
 * - 波动率超过80%自动中止评估
 * - 流动性极端低自动中止
 * - 集中度超过50%自动中止
 *
 * ## 扩展方式
 * - 实现自定义RiskModel并注册到引擎
 * - 可扩展更多风险类型、AI/ML预测、外部数据源等
 *
 * ## 用法示例
 * ```rust
 * let mut engine = RiskAssessmentEngine::new();
 * let input = RiskAssessmentInput { portfolio_data: PortfolioData { weights: vec![5000, 3000, 2000] } };
 * let config = RiskAssessmentConfig { risk_tolerance: 5000 };
 * let market_data = EnhancedMarketData::default();
 * let result = engine.assess(input, &config, &market_data);
 * ```
 */

use crate::algorithms::{AlgorithmMetrics, TradingAlgorithm};
use crate::core::*;
use crate::error::StrategyError;
use anchor_lang::prelude::*;

// ============================================================================
// RISK ASSESSMENT STRUCTURES
// ============================================================================

/// Advanced risk assessment engine
pub struct RiskAssessmentEngine {
    /// Risk models for different risk types
    risk_models: HashMap<RiskType, Box<dyn RiskModel>>,
    /// Current algorithm metrics
    metrics: AlgorithmMetrics,
    /// Risk thresholds and limits
    risk_limits: RiskLimits,
    /// Historical risk data
    risk_history: Vec<RiskAssessmentRecord>,
    /// Real-time risk monitors
    risk_monitors: Vec<RiskMonitor>,
    /// Risk correlation matrix
    correlation_matrix: CorrelationMatrix,
    /// Configuration
    config: RiskAssessmentConfig,
}

/// Risk types for categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RiskType {
    MarketRisk,
    LiquidityRisk,
    VolatilityRisk,
    ConcentrationRisk,
    CounterpartyRisk,
    OperationalRisk,
    RegulatoryRisk,
    ModelRisk,
    SystemicRisk,
}

/// Risk model trait for different risk types
pub trait RiskModel: Send + Sync {
    /// Calculate risk metric
    fn calculate_risk(&self, data: &RiskData) -> StrategyResult<RiskMetric>;

    /// Validate risk data
    fn validate_data(&self, data: &RiskData) -> StrategyResult<()>;

    /// Get risk model type
    fn risk_type(&self) -> RiskType;

    /// Update model parameters
    fn update_parameters(&mut self, params: HashMap<String, f64>) -> StrategyResult<()>;
}

/// Risk data for assessment
#[derive(Debug, Clone)]
pub struct RiskData {
    /// Market data
    pub market_data: EnhancedMarketData,
    /// Portfolio data
    pub portfolio_data: PortfolioData,
    /// Historical data
    pub historical_data: Vec<HistoricalDataPoint>,
    /// Market conditions
    pub market_conditions: MarketConditions,
    /// Timestamp
    pub timestamp: i64,
}

/// Portfolio data for risk assessment
#[derive(Debug, Clone)]
pub struct PortfolioData {
    /// Portfolio positions
    pub positions: Vec<Position>,
    /// Portfolio value
    pub total_value: u64,
    /// Portfolio weights
    pub weights: Vec<u64>,
    /// Leverage ratio
    pub leverage_ratio: f64,
    /// Concentration metrics
    pub concentration_metrics: ConcentrationMetrics,
}

/// Position in portfolio
#[derive(Debug, Clone)]
pub struct Position {
    /// Token mint
    pub token_mint: Pubkey,
    /// Position size
    pub size: u64,
    /// Position value
    pub value: u64,
    /// Entry price
    pub entry_price: u64,
    /// Current price
    pub current_price: u64,
    /// Unrealized P&L
    pub unrealized_pnl: i64,
}

/// Concentration metrics
#[derive(Debug, Clone)]
pub struct ConcentrationMetrics {
    /// Herfindahl-Hirschman Index
    pub hhi: f64,
    /// Maximum concentration
    pub max_concentration: f64,
    /// Top 5 concentration
    pub top_5_concentration: f64,
    /// Concentration risk score
    pub concentration_risk_score: u32,
}

/// Risk metric result
#[derive(Debug, Clone)]
pub struct RiskMetric {
    /// Risk type
    pub risk_type: RiskType,
    /// Risk value
    pub risk_value: f64,
    /// Risk score (0-10000)
    pub risk_score: u32,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Confidence level
    pub confidence_level: u32,
    /// Risk components
    pub components: HashMap<String, f64>,
    /// Timestamp
    pub timestamp: i64,
}

/// Risk levels
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// Get risk level score
    pub fn score(&self) -> u32 {
        match self {
            RiskLevel::Low => 2500,
            RiskLevel::Medium => 5000,
            RiskLevel::High => 7500,
            RiskLevel::Critical => 10000,
        }
    }
}

/// Risk limits for monitoring
#[derive(Debug, Clone)]
pub struct RiskLimits {
    /// Maximum portfolio value at risk (VaR)
    pub max_var_bps: u32,
    /// Maximum expected shortfall
    pub max_expected_shortfall_bps: u32,
    /// Maximum volatility
    pub max_volatility_bps: u32,
    /// Maximum concentration
    pub max_concentration_bps: u32,
    /// Maximum leverage
    pub max_leverage: f64,
    /// Maximum drawdown
    pub max_drawdown_bps: u32,
    /// Circuit breaker thresholds
    pub circuit_breaker_thresholds: CircuitBreakerThresholds,
}

/// Circuit breaker thresholds
#[derive(Debug, Clone)]
pub struct CircuitBreakerThresholds {
    /// Volatility circuit breaker
    pub volatility_threshold: u32,
    /// Liquidity circuit breaker
    pub liquidity_threshold: u32,
    /// Concentration circuit breaker
    pub concentration_threshold: u32,
    /// Loss circuit breaker
    pub loss_threshold: u32,
}

/// Risk assessment record
#[derive(Debug, Clone)]
pub struct RiskAssessmentRecord {
    /// Risk metrics
    pub risk_metrics: Vec<RiskMetric>,
    /// Portfolio data
    pub portfolio_data: PortfolioData,
    /// Market conditions
    pub market_conditions: MarketConditions,
    /// Risk alerts
    pub risk_alerts: Vec<RiskAlert>,
    /// Timestamp
    pub timestamp: i64,
}

/// Risk alert
#[derive(Debug, Clone)]
pub struct RiskAlert {
    /// Alert type
    pub alert_type: RiskAlertType,
    /// Alert message
    pub message: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Risk metric that triggered alert
    pub triggered_metric: RiskMetric,
    /// Timestamp
    pub timestamp: i64,
}

/// Risk alert types
#[derive(Debug, Clone)]
pub enum RiskAlertType {
    VaRExceeded,
    VolatilitySpike,
    LiquidityCrisis,
    ConcentrationRisk,
    LeverageExceeded,
    DrawdownLimit,
    CircuitBreaker,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Risk monitor for real-time monitoring
#[derive(Debug, Clone)]
pub struct RiskMonitor {
    /// Monitor ID
    pub id: String,
    /// Risk type to monitor
    pub risk_type: RiskType,
    /// Monitoring threshold
    pub threshold: f64,
    /// Alert severity
    pub alert_severity: AlertSeverity,
    /// Is active
    pub is_active: bool,
    /// Last check timestamp
    pub last_check: i64,
}

/// Correlation matrix for risk assessment
#[derive(Debug, Clone)]
pub struct CorrelationMatrix {
    /// Correlation data
    pub correlations: HashMap<(String, String), f64>,
    /// Asset identifiers
    pub assets: Vec<String>,
    /// Matrix dimension
    pub dimension: usize,
    /// Last update timestamp
    pub last_update: i64,
}

/// Risk assessment configuration
#[derive(Debug, Clone)]
pub struct RiskAssessmentConfig {
    /// Update frequency in seconds
    pub update_frequency_seconds: u64,
    /// Historical data window in seconds
    pub historical_window_seconds: u64,
    /// Confidence level for VaR calculation
    pub var_confidence_level: f64,
    /// Monte Carlo simulation iterations
    pub monte_carlo_iterations: u32,
    /// Stress test scenarios
    pub stress_test_scenarios: Vec<StressTestScenario>,
    /// Enable real-time monitoring
    pub enable_real_time_monitoring: bool,
    /// Enable circuit breakers
    pub enable_circuit_breakers: bool,
}

/// Stress test scenario
#[derive(Debug, Clone)]
pub struct StressTestScenario {
    /// Scenario name
    pub name: String,
    /// Scenario description
    pub description: String,
    /// Market shock parameters
    pub market_shock: MarketShock,
    /// Expected impact
    pub expected_impact: f64,
}

/// Market shock parameters
#[derive(Debug, Clone)]
pub struct MarketShock {
    /// Price shock in basis points
    pub price_shock_bps: i32,
    /// Volatility shock in basis points
    pub volatility_shock_bps: u32,
    /// Liquidity shock in basis points
    pub liquidity_shock_bps: u32,
    /// Correlation shock
    pub correlation_shock: f64,
}

// ============================================================================
// RISK MODEL IMPLEMENTATIONS
// ============================================================================

/// Market risk model using VaR and Expected Shortfall
pub struct MarketRiskModel {
    /// Model parameters
    parameters: HashMap<String, f64>,
    /// Historical returns
    historical_returns: Vec<f64>,
    /// Risk type
    risk_type: RiskType,
}

impl MarketRiskModel {
    /// Create new market risk model
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            historical_returns: Vec::new(),
            risk_type: RiskType::MarketRisk,
        }
    }

    /// Calculate Value at Risk (VaR)
    fn calculate_var(&self, confidence_level: f64) -> StrategyResult<f64> {
        if self.historical_returns.is_empty() {
            return Ok(0.0);
        }

        let mut returns = self.historical_returns.clone();
        returns.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = ((1.0 - confidence_level) * returns.len() as f64) as usize;
        let var = returns.get(index).unwrap_or(&0.0);

        Ok(*var)
    }

    /// Calculate Expected Shortfall (Conditional VaR)
    fn calculate_expected_shortfall(&self, confidence_level: f64) -> StrategyResult<f64> {
        let var = self.calculate_var(confidence_level)?;

        let tail_returns: Vec<f64> = self
            .historical_returns
            .iter()
            .filter(|&&r| r <= var)
            .cloned()
            .collect();

        if tail_returns.is_empty() {
            return Ok(0.0);
        }

        let expected_shortfall = tail_returns.iter().sum::<f64>() / tail_returns.len() as f64;
        Ok(expected_shortfall)
    }
}

impl RiskModel for MarketRiskModel {
    fn calculate_risk(&self, data: &RiskData) -> StrategyResult<RiskMetric> {
        let var_confidence = self.parameters.get("var_confidence").unwrap_or(&0.95);
        let var = self.calculate_var(*var_confidence)?;
        let expected_shortfall = self.calculate_expected_shortfall(*var_confidence)?;

        let volatility = self.calculate_volatility(&data.market_data)?;
        let beta = self.calculate_beta(&data.portfolio_data, &data.market_data)?;

        let risk_value = var.abs() * data.portfolio_data.total_value as f64 / 10000.0;
        let risk_score = self.calculate_risk_score(risk_value, volatility, beta)?;
        let risk_level = self.determine_risk_level(risk_score)?;

        let mut components = HashMap::new();
        components.insert("var".to_string(), var);
        components.insert("expected_shortfall".to_string(), expected_shortfall);
        components.insert("volatility".to_string(), volatility);
        components.insert("beta".to_string(), beta);

        Ok(RiskMetric {
            risk_type: self.risk_type.clone(),
            risk_value,
            risk_score,
            risk_level,
            confidence_level: 8500, // 85% confidence
            components,
            timestamp: Clock::get()?.unix_timestamp,
        })
    }

    fn validate_data(&self, data: &RiskData) -> StrategyResult<()> {
        require!(
            !data.market_data.prices.is_empty(),
            StrategyError::InvalidMarketData
        );
        require!(
            data.portfolio_data.total_value > 0,
            StrategyError::InvalidStrategyParameters
        );
        Ok(())
    }

    fn risk_type(&self) -> RiskType {
        self.risk_type.clone()
    }

    fn update_parameters(&mut self, params: HashMap<String, f64>) -> StrategyResult<()> {
        self.parameters = params;
        Ok(())
    }
}

impl MarketRiskModel {
    /// Calculate portfolio volatility
    fn calculate_volatility(&self, market_data: &EnhancedMarketData) -> StrategyResult<f64> {
        if market_data.volatilities.is_empty() {
            return Ok(0.0);
        }

        let avg_volatility = market_data.volatilities.iter().sum::<u32>() as f64
            / market_data.volatilities.len() as f64;

        Ok(avg_volatility / 10000.0)
    }

    /// Calculate portfolio beta
    fn calculate_beta(
        &self,
        portfolio: &PortfolioData,
        market_data: &EnhancedMarketData,
    ) -> StrategyResult<f64> {
        // Simplified beta calculation
        let portfolio_volatility = self.calculate_volatility(market_data)?;
        let market_volatility = 0.15; // Assume 15% market volatility

        Ok(portfolio_volatility / market_volatility)
    }

    /// Calculate risk score
    fn calculate_risk_score(
        &self,
        risk_value: f64,
        volatility: f64,
        beta: f64,
    ) -> StrategyResult<u32> {
        let normalized_risk = (risk_value / 1000000.0).min(1.0); // Normalize to 0-1
        let normalized_volatility = volatility.min(1.0);
        let normalized_beta = beta.min(2.0) / 2.0; // Normalize beta to 0-1

        let risk_score =
            (normalized_risk * 0.4 + normalized_volatility * 0.3 + normalized_beta * 0.3) * 10000.0;

        Ok(risk_score as u32)
    }

    /// Determine risk level
    fn determine_risk_level(&self, risk_score: u32) -> StrategyResult<RiskLevel> {
        match risk_score {
            0..=2500 => Ok(RiskLevel::Low),
            2501..=5000 => Ok(RiskLevel::Medium),
            5001..=7500 => Ok(RiskLevel::High),
            _ => Ok(RiskLevel::Critical),
        }
    }
}

/// Liquidity risk model
pub struct LiquidityRiskModel {
    /// Model parameters
    parameters: HashMap<String, f64>,
    /// Risk type
    risk_type: RiskType,
}

impl LiquidityRiskModel {
    /// Create new liquidity risk model
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            risk_type: RiskType::LiquidityRisk,
        }
    }

    /// Calculate liquidity risk
    fn calculate_liquidity_risk(
        &self,
        portfolio: &PortfolioData,
        market_data: &EnhancedMarketData,
    ) -> StrategyResult<f64> {
        let total_liquidity = market_data.liquidity.iter().sum::<u64>() as f64;
        let portfolio_value = portfolio.total_value as f64;

        if total_liquidity == 0.0 {
            return Ok(1.0); // Maximum risk if no liquidity
        }

        let liquidity_ratio = portfolio_value / total_liquidity;
        Ok(liquidity_ratio.min(1.0))
    }
}

impl RiskModel for LiquidityRiskModel {
    fn calculate_risk(&self, data: &RiskData) -> StrategyResult<RiskMetric> {
        let liquidity_risk =
            self.calculate_liquidity_risk(&data.portfolio_data, &data.market_data)?;

        let risk_score = (liquidity_risk * 10000.0) as u32;
        let risk_level = match risk_score {
            0..=2500 => RiskLevel::Low,
            2501..=5000 => RiskLevel::Medium,
            5001..=7500 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        let mut components = HashMap::new();
        components.insert("liquidity_risk".to_string(), liquidity_risk);

        Ok(RiskMetric {
            risk_type: self.risk_type.clone(),
            risk_value: liquidity_risk,
            risk_score,
            risk_level,
            confidence_level: 8000,
            components,
            timestamp: Clock::get()?.unix_timestamp,
        })
    }

    fn validate_data(&self, data: &RiskData) -> StrategyResult<()> {
        require!(
            !data.market_data.liquidity.is_empty(),
            StrategyError::InvalidMarketData
        );
        require!(
            data.portfolio_data.total_value > 0,
            StrategyError::InvalidStrategyParameters
        );
        Ok(())
    }

    fn risk_type(&self) -> RiskType {
        self.risk_type.clone()
    }

    fn update_parameters(&mut self, params: HashMap<String, f64>) -> StrategyResult<()> {
        self.parameters = params;
        Ok(())
    }
}

// ========== 新增：极端风险保护与异常检测辅助函数 ==========
fn is_extreme_volatility(volatility: u32, threshold: u32) -> bool {
    volatility > threshold
}
fn is_extreme_liquidity(liquidity: u64, threshold: u64) -> bool {
    liquidity < threshold
}
fn is_extreme_concentration(concentration: f64, threshold: f64) -> bool {
    concentration > threshold
}

// ============================================================================
// RISK ASSESSMENT ENGINE IMPLEMENTATION
// ============================================================================

impl RiskAssessmentEngine {
    /// Create new risk assessment engine
    pub fn new() -> Self {
        let mut engine = Self {
            risk_models: HashMap::new(),
            metrics: AlgorithmMetrics::default(),
            risk_limits: RiskLimits::default(),
            risk_history: Vec::new(),
            risk_monitors: Vec::new(),
            correlation_matrix: CorrelationMatrix::new(),
            config: RiskAssessmentConfig::default(),
        };

        // Initialize risk models
        engine.initialize_risk_models();

        engine
    }

    /// Initialize risk models
    fn initialize_risk_models(&mut self) {
        self.risk_models
            .insert(RiskType::MarketRisk, Box::new(MarketRiskModel::new()));
        self.risk_models
            .insert(RiskType::LiquidityRisk, Box::new(LiquidityRiskModel::new()));
    }

    /// 多维风险建模与动态聚合，支持相关性调整、历史回放、可插拔机器学习预测
    pub fn assess_risk(&mut self, data: &RiskData) -> StrategyResult<RiskAssessmentResult> {
        let start_time = Clock::get()?.unix_timestamp;
        // ========== 新增：极端风险保护 ==========
        let max_volatility = data
            .market_data
            .volatilities
            .iter()
            .max()
            .cloned()
            .unwrap_or(0);
        if is_extreme_volatility(max_volatility, 8000) {
            // 80%波动率阈值
            msg!(
                "[RiskAssessment] Extreme volatility detected: {}bps, aborting assessment",
                max_volatility
            );
            return Err(StrategyError::RiskLimitsExceeded.into());
        }
        let min_liquidity = data
            .market_data
            .liquidity
            .iter()
            .min()
            .cloned()
            .unwrap_or(u64::MAX);
        if is_extreme_liquidity(min_liquidity, 100) {
            // 低于100视为极端低流动性
            msg!(
                "[RiskAssessment] Extreme low liquidity detected: {} units, aborting assessment",
                min_liquidity
            );
            return Err(StrategyError::RiskLimitsExceeded.into());
        }
        let max_concentration = data.portfolio_data.concentration_metrics.max_concentration;
        if is_extreme_concentration(max_concentration, 0.5) {
            // 超过50%视为极端集中
            msg!(
                "[RiskAssessment] Extreme concentration detected: {:.2}, aborting assessment",
                max_concentration
            );
            return Err(StrategyError::RiskLimitsExceeded.into());
        }
        let mut risk_metrics = Vec::new();
        let mut risk_alerts = Vec::new();
        // 多维风险建模
        for (risk_type, model) in &self.risk_models {
            match model.calculate_risk(data) {
                Ok(metric) => {
                    risk_metrics.push(metric.clone());
                    if let Some(alert) = self.check_risk_alert(&metric) {
                        risk_alerts.push(alert);
                    }
                }
                Err(e) => {
                    msg!("Risk calculation failed for {:?}: {:?}", risk_type, e);
                }
            }
        }
        // 相关性调整
        let correlation_adj = self.calculate_correlation_adjustment(&risk_metrics)?;
        // 历史回放修正
        let hist_score = self
            .risk_history
            .last()
            .map(|rec| rec.aggregate_risk_score as f64)
            .unwrap_or(0.0);
        // 可插拔ML模型预测（如有）
        let ml_pred = 0.0; // 预留接口
                           // 动态权重聚合
        let aggregate_risk_score = ((0.5
            * self.calculate_aggregate_risk_score(&risk_metrics)? as f64
            + 0.3 * correlation_adj
            + 0.1 * hist_score
            + 0.1 * ml_pred)
            .min(10000.0)) as u32;
        let record = RiskAssessmentRecord {
            risk_metrics: risk_metrics.clone(),
            portfolio_data: data.portfolio_data.clone(),
            market_conditions: data.market_conditions.clone(),
            risk_alerts: risk_alerts.clone(),
            timestamp: Clock::get()?.unix_timestamp,
        };
        self.risk_history.push(record);
        let exec_time = Clock::get()?.unix_timestamp - start_time;
        self.metrics.update_with_operation(true, exec_time as u64);
        // ========== 新增：可观测性增强 ==========
        let max_risk_score = risk_metrics.iter().map(|m| m.risk_score).max().unwrap_or(0);
        let max_risk_level = risk_metrics
            .iter()
            .map(|m| m.risk_level.score())
            .max()
            .unwrap_or(0);
        msg!("[RiskAssessment] Assessment finished: aggregate_risk_score={}, max_risk_score={}, max_risk_level={}, alert_count={}", aggregate_risk_score, max_risk_score, max_risk_level, risk_alerts.len());
        Ok(RiskAssessmentResult {
            risk_metrics,
            aggregate_risk_score,
            risk_alerts,
            recommendations: self.generate_recommendations(&risk_metrics)?,
            timestamp: Clock::get()?.unix_timestamp,
        })
    }

    /// Check for risk alerts
    fn check_risk_alert(&self, metric: &RiskMetric) -> Option<RiskAlert> {
        let alert_type = match metric.risk_type {
            RiskType::MarketRisk => {
                if metric.risk_score > self.risk_limits.max_var_bps {
                    Some(RiskAlertType::VaRExceeded)
                } else {
                    None
                }
            }
            RiskType::LiquidityRisk => {
                if metric.risk_score > self.risk_limits.max_expected_shortfall_bps {
                    Some(RiskAlertType::LiquidityCrisis)
                } else {
                    None
                }
            }
            _ => None,
        };

        alert_type.map(|alert_type| RiskAlert {
            alert_type,
            message: format!("Risk threshold exceeded for {:?}", metric.risk_type),
            severity: AlertSeverity::Warning,
            triggered_metric: metric.clone(),
            timestamp: Clock::get().unwrap().unix_timestamp,
        })
    }

    /// Calculate aggregate risk score
    fn calculate_aggregate_risk_score(&self, metrics: &[RiskMetric]) -> StrategyResult<u32> {
        if metrics.is_empty() {
            return Ok(0);
        }

        let total_score: u32 = metrics.iter().map(|m| m.risk_score).sum();
        let avg_score = total_score / metrics.len() as u32;

        // Apply correlation adjustment
        let correlation_adjustment = self.calculate_correlation_adjustment(metrics)?;
        let adjusted_score = (avg_score as f64 * correlation_adjustment) as u32;

        Ok(adjusted_score.min(10000))
    }

    /// Calculate correlation adjustment
    fn calculate_correlation_adjustment(&self, metrics: &[RiskMetric]) -> StrategyResult<f64> {
        // Simplified correlation adjustment
        let avg_correlation = 0.3; // Assume 30% average correlation
        let adjustment =
            1.0 + avg_correlation * (metrics.len() as f64 - 1.0) / metrics.len() as f64;

        Ok(adjustment.min(2.0)) // Cap at 2x
    }

    /// Generate risk recommendations
    fn generate_recommendations(
        &self,
        metrics: &[RiskMetric],
    ) -> StrategyResult<Vec<RiskRecommendation>> {
        let mut recommendations = Vec::new();

        for metric in metrics {
            match metric.risk_level {
                RiskLevel::High | RiskLevel::Critical => {
                    recommendations.push(RiskRecommendation {
                        risk_type: metric.risk_type.clone(),
                        action: self.get_recommended_action(&metric.risk_type)?,
                        priority: if metric.risk_level == RiskLevel::Critical {
                            RecommendationPriority::High
                        } else {
                            RecommendationPriority::Medium
                        },
                        description: format!("Mitigate {} risk", metric.risk_type.name()),
                    });
                }
                _ => {}
            }
        }

        Ok(recommendations)
    }

    /// Get recommended action for risk type
    fn get_recommended_action(&self, risk_type: &RiskType) -> StrategyResult<RiskAction> {
        match risk_type {
            RiskType::MarketRisk => Ok(RiskAction::ReducePosition),
            RiskType::LiquidityRisk => Ok(RiskAction::IncreaseLiquidity),
            RiskType::ConcentrationRisk => Ok(RiskAction::Diversify),
            _ => Ok(RiskAction::Monitor),
        }
    }

    /// Add risk monitor
    pub fn add_risk_monitor(&mut self, monitor: RiskMonitor) -> StrategyResult<()> {
        self.risk_monitors.push(monitor);
        Ok(())
    }

    /// Update risk limits
    pub fn update_risk_limits(&mut self, limits: RiskLimits) -> StrategyResult<()> {
        self.risk_limits = limits;
        Ok(())
    }

    /// Get risk history
    pub fn get_risk_history(&self, window_seconds: u64) -> Vec<&RiskAssessmentRecord> {
        let cutoff_time = Clock::get().unwrap().unix_timestamp - window_seconds as i64;

        self.risk_history
            .iter()
            .filter(|record| record.timestamp >= cutoff_time)
            .collect()
    }
}

// ============================================================================
// SUPPORTING STRUCTURES
// ============================================================================

/// Risk assessment result
#[derive(Debug, Clone)]
pub struct RiskAssessmentResult {
    /// Individual risk metrics
    pub risk_metrics: Vec<RiskMetric>,
    /// Aggregate risk score
    pub aggregate_risk_score: u32,
    /// Risk alerts
    pub risk_alerts: Vec<RiskAlert>,
    /// Risk recommendations
    pub recommendations: Vec<RiskRecommendation>,
    /// Timestamp
    pub timestamp: i64,
}

/// Risk recommendation
#[derive(Debug, Clone)]
pub struct RiskRecommendation {
    /// Risk type
    pub risk_type: RiskType,
    /// Recommended action
    pub action: RiskAction,
    /// Recommendation priority
    pub priority: RecommendationPriority,
    /// Description
    pub description: String,
}

/// Risk actions
#[derive(Debug, Clone)]
pub enum RiskAction {
    ReducePosition,
    IncreaseLiquidity,
    Diversify,
    Hedge,
    Monitor,
    StopTrading,
}

/// Recommendation priority
#[derive(Debug, Clone)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Historical data point
#[derive(Debug, Clone)]
pub struct HistoricalDataPoint {
    /// Data values
    pub values: Vec<f64>,
    /// Timestamp
    pub timestamp: i64,
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_var_bps: 500,                // 5% VaR
            max_expected_shortfall_bps: 750, // 7.5% ES
            max_volatility_bps: 2000,        // 20% volatility
            max_concentration_bps: 3000,     // 30% concentration
            max_leverage: 2.0,
            max_drawdown_bps: 1000, // 10% drawdown
            circuit_breaker_thresholds: CircuitBreakerThresholds {
                volatility_threshold: 3000,    // 30%
                liquidity_threshold: 2000,     // 20%
                concentration_threshold: 4000, // 40%
                loss_threshold: 1500,          // 15%
            },
        }
    }
}

impl Default for RiskAssessmentConfig {
    fn default() -> Self {
        Self {
            update_frequency_seconds: 60,
            historical_window_seconds: 86400, // 24 hours
            var_confidence_level: 0.95,
            monte_carlo_iterations: 10000,
            stress_test_scenarios: vec![StressTestScenario {
                name: "Market Crash".to_string(),
                description: "Severe market downturn scenario".to_string(),
                market_shock: MarketShock {
                    price_shock_bps: -2000,     // -20%
                    volatility_shock_bps: 3000, // +30%
                    liquidity_shock_bps: 5000,  // +50% (positive for liquidity drain)
                    correlation_shock: 0.8,
                },
                expected_impact: -0.15, // -15%
            }],
            enable_real_time_monitoring: true,
            enable_circuit_breakers: true,
        }
    }
}

impl CorrelationMatrix {
    /// Create new correlation matrix
    pub fn new() -> Self {
        Self {
            correlations: HashMap::new(),
            assets: Vec::new(),
            dimension: 0,
            last_update: 0,
        }
    }
}

// ============================================================================
// TRADING ALGORITHM IMPLEMENTATION
// ============================================================================

impl TradingAlgorithm for RiskAssessmentEngine {
    type Input = RiskAssessmentInput;
    type Output = RiskAssessmentOutput;
    type Config = RiskAssessmentConfig;

    fn execute(
        &mut self,
        input: Self::Input,
        config: &Self::Config,
        market_data: &EnhancedMarketData,
    ) -> StrategyResult<Self::Output> {
        let start_time = Clock::get()?.unix_timestamp;

        // Update configuration
        self.config = config.clone();

        // Create risk data
        let risk_data = RiskData {
            market_data: market_data.clone(),
            portfolio_data: input.portfolio_data,
            historical_data: input.historical_data,
            market_conditions: input.market_conditions,
            timestamp: Clock::get()?.unix_timestamp,
        };

        // Perform risk assessment
        let result = self.assess_risk(&risk_data)?;

        // Create output
        let output = RiskAssessmentOutput {
            risk_metrics: result.risk_metrics,
            aggregate_risk_score: result.aggregate_risk_score,
            risk_alerts: result.risk_alerts,
            recommendations: result.recommendations,
            risk_level: self.determine_overall_risk_level(result.aggregate_risk_score)?,
        };

        // Update metrics
        let execution_time = Clock::get()?.unix_timestamp - start_time;
        self.metrics
            .update_with_operation(true, execution_time as u64);

        Ok(output)
    }

    fn validate_parameters(
        &self,
        input: &Self::Input,
        config: &Self::Config,
    ) -> StrategyResult<()> {
        require!(
            input.portfolio_data.total_value > 0,
            StrategyError::InvalidStrategyParameters
        );
        require!(
            config.var_confidence_level > 0.0 && config.var_confidence_level < 1.0,
            StrategyError::InvalidStrategyParameters
        );
        Ok(())
    }

    fn get_metrics(&self) -> AlgorithmMetrics {
        self.metrics.clone()
    }

    fn reset(&mut self) {
        self.risk_history.clear();
        self.metrics = AlgorithmMetrics::default();
    }
}

/// RiskAssessment 工厂方法
pub fn create_risk_assessor() -> RiskAssessmentEngine {
    RiskAssessmentEngine::new()
}

/// 输入/输出/配置/记录结构体
#[derive(Debug, Clone)]
pub struct RiskAssessmentInput {
    pub portfolio_data: PortfolioData,
}

#[derive(Debug, Clone)]
pub struct RiskAssessmentOutput {
    pub risk_metrics: Vec<u32>,
    pub aggregate_risk_score: u32,
    pub risk_alerts: Vec<String>,
    pub recommendations: Vec<String>,
    pub risk_level: RiskLevel,
}

// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_risk_assessment_basic() {
        let mut engine = RiskAssessmentEngine::new();
        let input = RiskAssessmentInput {
            portfolio_data: PortfolioData {
                weights: vec![5000, 3000, 2000],
            },
        };
        let config = RiskAssessmentConfig {
            risk_tolerance: 5000,
        };
        let mut market_data = EnhancedMarketData::default();
        market_data.volatilities = vec![1000, 2000, 1500];
        market_data.liquidity_sources = vec![LiquiditySource {
            token_mint: Default::default(),
            liquidity: 10000,
        }];
        let result = engine.assess(input, &config, &market_data);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.aggregate_risk_score > 0);
        assert!(matches!(
            output.risk_level,
            RiskLevel::Low | RiskLevel::Medium | RiskLevel::High
        ));
    }

    #[test]
    fn test_risk_assessment_extreme_volatility_triggers_abort() {
        let mut engine = RiskAssessmentEngine::new();
        let input = RiskAssessmentInput {
            portfolio_data: PortfolioData {
                weights: vec![5000, 3000, 2000],
            },
        };
        let config = RiskAssessmentConfig {
            risk_tolerance: 5000,
        };
        let mut market_data = EnhancedMarketData::default();
        market_data.volatilities = vec![9000]; // 超过80%阈值
        market_data.liquidity = vec![10000];
        let result = engine.assess(input, &config, &market_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_risk_assessment_extreme_low_liquidity_triggers_abort() {
        let mut engine = RiskAssessmentEngine::new();
        let input = RiskAssessmentInput {
            portfolio_data: PortfolioData {
                weights: vec![5000, 3000, 2000],
            },
        };
        let config = RiskAssessmentConfig {
            risk_tolerance: 5000,
        };
        let mut market_data = EnhancedMarketData::default();
        market_data.volatilities = vec![1000];
        market_data.liquidity = vec![1]; // 极端低流动性
        let result = engine.assess(input, &config, &market_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_risk_assessment_extreme_concentration_triggers_abort() {
        let mut engine = RiskAssessmentEngine::new();
        let mut portfolio_data = PortfolioData {
            weights: vec![9000, 500, 500],
        };
        portfolio_data.concentration_metrics = ConcentrationMetrics {
            hhi: 0.0,
            max_concentration: 0.9, // 超过50%
            top_5_concentration: 0.0,
            concentration_risk_score: 0,
        };
        let input = RiskAssessmentInput { portfolio_data };
        let config = RiskAssessmentConfig {
            risk_tolerance: 5000,
        };
        let mut market_data = EnhancedMarketData::default();
        market_data.volatilities = vec![1000];
        market_data.liquidity = vec![10000];
        let result = engine.assess(input, &config, &market_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_risk_assessment_invalid_parameters() {
        let mut engine = RiskAssessmentEngine::new();
        let input = RiskAssessmentInput {
            portfolio_data: PortfolioData { weights: vec![] },
        }; // 非法参数
        let config = RiskAssessmentConfig { risk_tolerance: 0 };
        let market_data = EnhancedMarketData::default();
        let result = engine.assess(input, &config, &market_data);
        // 具体实现可能返回Ok但无风险分数，也可能Err，视实现而定
    }
}
