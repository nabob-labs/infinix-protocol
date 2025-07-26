//!
//! validatable.rs - 类型校验trait与实现
//!
//! 本文件定义了Validatable trait及其对RiskMetrics、MarketData、TokenInfo、WeightAllocation的实现，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::core::types::risk::RiskMetrics;
use crate::core::types::market::MarketData;
use crate::core::types::token::{TokenInfo, WeightAllocation};

/// 类型校验trait
pub trait Validatable {
    /// 验证类型并返回 Result
    fn validate(&self) -> Result<()>;
}

impl Validatable for RiskMetrics {
    fn validate(&self) -> Result<()> {
        require!(self.var_95 <= 10000 && self.var_99 <= 10000, crate::error::ErrorCode::InvalidParams);
        Ok(())
    }
}

impl Validatable for MarketData {
    fn validate(&self) -> Result<()> {
        require!(self.price > 0, crate::error::ErrorCode::InvalidParams);
        Ok(())
    }
}

impl Validatable for TokenInfo {
    fn validate(&self) -> Result<()> {
        require!(!self.symbol.is_empty(), crate::error::ErrorCode::InvalidParams);
        Ok(())
    }
}

impl Validatable for WeightAllocation {
    fn validate(&self) -> Result<()> {
        require!(self.weight_bps <= 10000, crate::error::ErrorCode::InvalidParams);
        Ok(())
    }
} 