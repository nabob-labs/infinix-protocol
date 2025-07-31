//!
//! traits.rs - 通用校验Trait与注册表定义
//!
//! 本文件定义了通用校验Trait与注册表，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use anchor_lang::prelude::*;
use crate::errors::strategy_error::StrategyError;

/// 通用校验 trait
/// - 支持参数/状态/业务等多场景可插拔校验器
pub trait Validator<T>: Send + Sync {
    /// 校验 value 是否满足约束，失败返回 Err
    fn validate(&self, value: &T) -> anchor_lang::Result<()>;
}

/// 校验器注册表
/// - 支持批量注册和统一校验
pub struct ValidatorRegistry<T> {
    validators: Vec<Box<dyn Validator<T>>>,
}

impl<T> ValidatorRegistry<T> {
    /// 创建新注册表
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }
    /// 注册校验器
    pub fn register(&mut self, validator: Box<dyn Validator<T>>) {
        self.validators.push(validator);
    }
    /// 依次校验所有注册校验器
    pub fn validate_all(&self, value: &T) -> anchor_lang::Result<()> {
        for v in &self.validators {
            v.validate(value)?;
        }
        Ok(())
    }
}

/// 非空校验器
pub struct NotEmptyValidator;
impl<T: AsRef<[U]>, U> Validator<T> for NotEmptyValidator {
    fn validate(&self, value: &T) -> anchor_lang::Result<()> {
        if value.as_ref().is_empty() {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 数值范围校验器
pub struct RangeValidator {
    pub min: u64,
    pub max: u64,
}
impl Validator<u64> for RangeValidator {
    fn validate(&self, value: &u64) -> anchor_lang::Result<()> {
        if *value < self.min || *value > self.max {
            return Err(StrategyError::InvalidStrategyParameters.into());
        }
        Ok(())
    }
}

/// 权重和校验器（业务约束）
pub struct WeightsSumValidator;
impl Validator<Vec<u64>> for WeightsSumValidator {
    fn validate(&self, value: &Vec<u64>) -> anchor_lang::Result<()> {
        if value.iter().sum::<u64>() != 10_000 {
            return Err(StrategyError::InvalidWeightSum.into());
        }
        Ok(())
    }
} 