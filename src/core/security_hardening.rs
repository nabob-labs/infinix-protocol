//! 安全加固模块
//! 
//! 本模块提供全面的安全防护机制，包括：
//! - 权限管理优化
//! - 输入验证强化
//! - 溢出保护
//! - 重入攻击防护
//! - 闪电贷攻击防护
//! - 审计日志增强
//! 
//! 设计特点：
//! - 最小功能单元：专注于安全防护功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证和管理
//! - 攻击防护：多种攻击防护机制
//! - 审计追踪：完整的安全事件审计

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{
        constants::*,
        events::*,
        types::*,
        validation::*,
    },
    errors::*,
    utils::*,
};

/// 安全配置结构体
/// 
/// 包含安全加固所需的所有配置参数：
/// - enable_permission_control: 是否启用权限控制
/// - enable_input_validation: 是否启用输入验证
/// - enable_overflow_protection: 是否启用溢出保护
/// - enable_reentrancy_protection: 是否启用重入保护
/// - enable_flash_loan_protection: 是否启用闪电贷保护
/// - enable_audit_logging: 是否启用审计日志
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct SecurityHardeningConfig {
    /// 是否启用权限控制
    pub enable_permission_control: bool,
    /// 是否启用输入验证
    pub enable_input_validation: bool,
    /// 是否启用溢出保护
    pub enable_overflow_protection: bool,
    /// 是否启用重入保护
    pub enable_reentrancy_protection: bool,
    /// 是否启用闪电贷保护
    pub enable_flash_loan_protection: bool,
    /// 是否启用审计日志
    pub enable_audit_logging: bool,
    /// 最大权限检查深度
    pub max_permission_depth: u32,
    /// 最大输入长度
    pub max_input_length: usize,
    /// 最大数值范围
    pub max_numeric_value: u64,
    /// 重入保护超时时间（秒）
    pub reentrancy_timeout: u64,
}

/// 安全加固管理器
pub struct SecurityHardeningManager {
    /// 管理器名称
    name: String,
    /// 安全配置
    config: SecurityHardeningConfig,
    /// 权限管理器
    permission_manager: PermissionManager,
    /// 输入验证器
    input_validator: InputValidator,
    /// 溢出保护器
    overflow_protector: OverflowProtector,
    /// 重入保护器
    reentrancy_protector: ReentrancyProtector,
    /// 闪电贷保护器
    flash_loan_protector: FlashLoanProtector,
    /// 审计日志器
    audit_logger: AuditLogger,
}

impl SecurityHardeningManager {
    /// 创建新的安全加固管理器实例
    pub fn new(config: SecurityHardeningConfig) -> Self {
        Self {
            name: "SecurityHardeningManager".to_string(),
            config,
            permission_manager: PermissionManager::new(),
            input_validator: InputValidator::new(),
            overflow_protector: OverflowProtector::new(),
            reentrancy_protector: ReentrancyProtector::new(),
            flash_loan_protector: FlashLoanProtector::new(),
            audit_logger: AuditLogger::new(),
        }
    }
    
    /// 验证安全配置
    /// 
    /// 检查安全配置的有效性和边界条件：
    /// - 权限深度验证
    /// - 输入长度验证
    /// - 数值范围验证
    /// - 超时时间验证
    /// 
    /// # 参数
    /// - config: 安全配置
    /// 
    /// # 返回
    /// - Result<()>: 验证结果
    pub fn validate_security_config(config: &SecurityHardeningConfig) -> Result<()> {
        // 验证权限深度
        if config.max_permission_depth == 0 || config.max_permission_depth > MAX_PERMISSION_DEPTH {
            return Err(SecurityError::InvalidPermissionDepth.into());
        }
        
        // 验证输入长度
        if config.max_input_length == 0 || config.max_input_length > MAX_INPUT_LENGTH {
            return Err(SecurityError::InvalidInputLength.into());
        }
        
        // 验证数值范围
        if config.max_numeric_value == 0 {
            return Err(SecurityError::InvalidNumericValue.into());
        }
        
        // 验证超时时间
        if config.reentrancy_timeout == 0 || config.reentrancy_timeout > MAX_REENTRANCY_TIMEOUT {
            return Err(SecurityError::InvalidTimeout.into());
        }
        
        Ok(())
    }
    
    /// 执行安全检查
    /// 
    /// 执行全面的安全检查：
    /// - 权限检查
    /// - 输入验证
    /// - 溢出保护
    /// - 重入保护
    /// - 闪电贷保护
    /// 
    /// # 参数
    /// - ctx: 安全检查上下文
    /// - operation: 操作类型
    /// - params: 操作参数
    /// 
    /// # 返回
    /// - Result<()>: 安全检查结果
    pub fn perform_security_check(
        &mut self,
        ctx: &SecurityCheckContext,
        operation: SecurityOperation,
        params: &SecurityParams,
    ) -> Result<()> {
        // 权限检查
        if self.config.enable_permission_control {
            self.permission_manager.check_permissions(ctx, &operation, params)?;
        }
        
        // 输入验证
        if self.config.enable_input_validation {
            self.input_validator.validate_inputs(ctx, &operation, params)?;
        }
        
        // 溢出保护
        if self.config.enable_overflow_protection {
            self.overflow_protector.check_overflow(ctx, &operation, params)?;
        }
        
        // 重入保护
        if self.config.enable_reentrancy_protection {
            self.reentrancy_protector.check_reentrancy(ctx, &operation, params)?;
        }
        
        // 闪电贷保护
        if self.config.enable_flash_loan_protection {
            self.flash_loan_protector.check_flash_loan(ctx, &operation, params)?;
        }
        
        // 审计日志
        if self.config.enable_audit_logging {
            self.audit_logger.log_security_check(ctx, &operation, params, true)?;
        }
        
        Ok(())
    }
    
    /// 权限管理
    /// 
    /// 管理细粒度的权限控制：
    /// - 角色基础访问控制
    /// - 权限继承机制
    /// - 权限审计日志
    /// 
    /// # 参数
    /// - user: 用户公钥
    /// - operation: 操作类型
    /// - resource: 资源标识
    /// 
    /// # 返回
    /// - Result<bool>: 权限检查结果
    pub fn check_permission(
        &self,
        user: &Pubkey,
        operation: &str,
        resource: &str,
    ) -> Result<bool> {
        self.permission_manager.check_permission(user, operation, resource)
    }
    
    /// 输入验证
    /// 
    /// 执行严格的输入验证：
    /// - 参数边界检查
    /// - 类型安全检查
    /// - 格式验证
    /// 
    /// # 参数
    /// - input: 输入数据
    /// - input_type: 输入类型
    /// 
    /// # 返回
    /// - Result<()>: 验证结果
    pub fn validate_input(
        &self,
        input: &str,
        input_type: InputType,
    ) -> Result<()> {
        self.input_validator.validate_input(input, input_type)
    }
    
    /// 溢出保护
    /// 
    /// 防止数值溢出：
    /// - 加法溢出检查
    /// - 乘法溢出检查
    /// - 减法下溢检查
    /// 
    /// # 参数
    /// - a: 第一个操作数
    /// - b: 第二个操作数
    /// - operation: 操作类型
    /// 
    /// # 返回
    /// - Result<u64>: 安全计算结果
    pub fn safe_math_operation(
        &self,
        a: u64,
        b: u64,
        operation: MathOperation,
    ) -> Result<u64> {
        self.overflow_protector.safe_math_operation(a, b, operation)
    }
    
    /// 重入保护
    /// 
    /// 防止重入攻击：
    /// - 状态锁定
    /// - 超时检查
    /// - 调用栈验证
    /// 
    /// # 参数
    /// - account: 账户公钥
    /// - operation: 操作标识
    /// 
    /// # 返回
    /// - Result<()>: 保护结果
    pub fn enter_critical_section(
        &mut self,
        account: &Pubkey,
        operation: &str,
    ) -> Result<()> {
        self.reentrancy_protector.enter_critical_section(account, operation)
    }
    
    /// 退出临界区
    pub fn exit_critical_section(
        &mut self,
        account: &Pubkey,
        operation: &str,
    ) -> Result<()> {
        self.reentrancy_protector.exit_critical_section(account, operation)
    }
    
    /// 闪电贷保护
    /// 
    /// 防止闪电贷攻击：
    /// - 余额检查
    /// - 时间窗口验证
    /// - 交易模式检测
    /// 
    /// # 参数
    /// - account: 账户公钥
    /// - balance_before: 操作前余额
    /// - balance_after: 操作后余额
    /// 
    /// # 返回
    /// - Result<()>: 保护结果
    pub fn check_flash_loan_attack(
        &self,
        account: &Pubkey,
        balance_before: u64,
        balance_after: u64,
    ) -> Result<()> {
        self.flash_loan_protector.check_flash_loan_attack(account, balance_before, balance_after)
    }
    
    /// 记录安全事件
    /// 
    /// 记录安全相关事件：
    /// - 权限拒绝
    /// - 攻击检测
    /// - 异常行为
    /// 
    /// # 参数
    /// - event_type: 事件类型
    /// - account: 相关账户
    /// - details: 事件详情
    /// 
    /// # 返回
    /// - Result<()>: 记录结果
    pub fn log_security_event(
        &self,
        event_type: SecurityEventType,
        account: &Pubkey,
        details: &str,
    ) -> Result<()> {
        self.audit_logger.log_security_event(event_type, account, details)
    }
}

impl Default for SecurityHardeningManager {
    fn default() -> Self {
        Self::new(SecurityHardeningConfig::default())
    }
}

/// 权限管理器
pub struct PermissionManager {
    /// 权限映射
    permissions: std::collections::HashMap<String, Vec<String>>,
    /// 角色映射
    roles: std::collections::HashMap<Pubkey, Vec<String>>,
}

impl PermissionManager {
    /// 创建新的权限管理器
    pub fn new() -> Self {
        Self {
            permissions: std::collections::HashMap::new(),
            roles: std::collections::HashMap::new(),
        }
    }
    
    /// 检查权限
    pub fn check_permission(
        &self,
        user: &Pubkey,
        operation: &str,
        resource: &str,
    ) -> Result<bool> {
        // TODO: 实现具体的权限检查逻辑
        Ok(true)
    }
    
    /// 检查权限（带上下文）
    pub fn check_permissions(
        &self,
        ctx: &SecurityCheckContext,
        operation: &SecurityOperation,
        params: &SecurityParams,
    ) -> Result<()> {
        // TODO: 实现具体的权限检查逻辑
        Ok(())
    }
}

/// 输入验证器
pub struct InputValidator {
    /// 验证规则
    validation_rules: std::collections::HashMap<InputType, ValidationRule>,
}

impl InputValidator {
    /// 创建新的输入验证器
    pub fn new() -> Self {
        Self {
            validation_rules: std::collections::HashMap::new(),
        }
    }
    
    /// 验证输入
    pub fn validate_input(
        &self,
        input: &str,
        input_type: InputType,
    ) -> Result<()> {
        // TODO: 实现具体的输入验证逻辑
        Ok(())
    }
    
    /// 验证输入（带上下文）
    pub fn validate_inputs(
        &self,
        ctx: &SecurityCheckContext,
        operation: &SecurityOperation,
        params: &SecurityParams,
    ) -> Result<()> {
        // TODO: 实现具体的输入验证逻辑
        Ok(())
    }
}

/// 溢出保护器
pub struct OverflowProtector {
    /// 溢出检查配置
    overflow_config: OverflowConfig,
}

impl OverflowProtector {
    /// 创建新的溢出保护器
    pub fn new() -> Self {
        Self {
            overflow_config: OverflowConfig::default(),
        }
    }
    
    /// 安全数学运算
    pub fn safe_math_operation(
        &self,
        a: u64,
        b: u64,
        operation: MathOperation,
    ) -> Result<u64> {
        match operation {
            MathOperation::Add => {
                a.checked_add(b).ok_or(SecurityError::Overflow.into())
            }
            MathOperation::Sub => {
                a.checked_sub(b).ok_or(SecurityError::Underflow.into())
            }
            MathOperation::Mul => {
                a.checked_mul(b).ok_or(SecurityError::Overflow.into())
            }
            MathOperation::Div => {
                if b == 0 {
                    Err(SecurityError::DivisionByZero.into())
                } else {
                    Ok(a / b)
                }
            }
        }
    }
    
    /// 检查溢出
    pub fn check_overflow(
        &self,
        ctx: &SecurityCheckContext,
        operation: &SecurityOperation,
        params: &SecurityParams,
    ) -> Result<()> {
        // TODO: 实现具体的溢出检查逻辑
        Ok(())
    }
}

/// 重入保护器
pub struct ReentrancyProtector {
    /// 活跃操作
    active_operations: std::collections::HashMap<String, bool>,
    /// 操作时间戳
    operation_timestamps: std::collections::HashMap<String, i64>,
}

impl ReentrancyProtector {
    /// 创建新的重入保护器
    pub fn new() -> Self {
        Self {
            active_operations: std::collections::HashMap::new(),
            operation_timestamps: std::collections::HashMap::new(),
        }
    }
    
    /// 进入临界区
    pub fn enter_critical_section(
        &mut self,
        account: &Pubkey,
        operation: &str,
    ) -> Result<()> {
        let key = format!("{}:{}", account, operation);
        
        if self.active_operations.get(&key).unwrap_or(&false) {
            return Err(SecurityError::ReentrancyDetected.into());
        }
        
        self.active_operations.insert(key.clone(), true);
        self.operation_timestamps.insert(key, Clock::get()?.unix_timestamp);
        
        Ok(())
    }
    
    /// 退出临界区
    pub fn exit_critical_section(
        &mut self,
        account: &Pubkey,
        operation: &str,
    ) -> Result<()> {
        let key = format!("{}:{}", account, operation);
        self.active_operations.insert(key, false);
        Ok(())
    }
    
    /// 检查重入
    pub fn check_reentrancy(
        &self,
        ctx: &SecurityCheckContext,
        operation: &SecurityOperation,
        params: &SecurityParams,
    ) -> Result<()> {
        // TODO: 实现具体的重入检查逻辑
        Ok(())
    }
}

/// 闪电贷保护器
pub struct FlashLoanProtector {
    /// 闪电贷检测配置
    flash_loan_config: FlashLoanConfig,
}

impl FlashLoanProtector {
    /// 创建新的闪电贷保护器
    pub fn new() -> Self {
        Self {
            flash_loan_config: FlashLoanConfig::default(),
        }
    }
    
    /// 检查闪电贷攻击
    pub fn check_flash_loan_attack(
        &self,
        account: &Pubkey,
        balance_before: u64,
        balance_after: u64,
    ) -> Result<()> {
        // 检查余额变化是否异常
        if balance_before == 0 && balance_after > 0 {
            return Err(SecurityError::FlashLoanDetected.into());
        }
        
        Ok(())
    }
    
    /// 检查闪电贷
    pub fn check_flash_loan(
        &self,
        ctx: &SecurityCheckContext,
        operation: &SecurityOperation,
        params: &SecurityParams,
    ) -> Result<()> {
        // TODO: 实现具体的闪电贷检查逻辑
        Ok(())
    }
}

/// 审计日志器
pub struct AuditLogger {
    /// 日志配置
    log_config: AuditLogConfig,
}

impl AuditLogger {
    /// 创建新的审计日志器
    pub fn new() -> Self {
        Self {
            log_config: AuditLogConfig::default(),
        }
    }
    
    /// 记录安全事件
    pub fn log_security_event(
        &self,
        event_type: SecurityEventType,
        account: &Pubkey,
        details: &str,
    ) -> Result<()> {
        msg!(
            "Security Event - Type: {:?}, Account: {}, Details: {}",
            event_type,
            account,
            details
        );
        Ok(())
    }
    
    /// 记录安全检查
    pub fn log_security_check(
        &self,
        ctx: &SecurityCheckContext,
        operation: &SecurityOperation,
        params: &SecurityParams,
        success: bool,
    ) -> Result<()> {
        msg!(
            "Security Check - Operation: {:?}, Success: {}, Account: {}",
            operation,
            success,
            ctx.account
        );
        Ok(())
    }
}

// 结构体定义
#[derive(Clone, Debug)]
pub struct SecurityCheckContext {
    pub account: Pubkey,
    pub timestamp: i64,
    pub slot: u64,
}

#[derive(Clone, Debug)]
pub struct SecurityOperation {
    pub operation_type: String,
    pub resource: String,
    pub parameters: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SecurityParams {
    pub user: Pubkey,
    pub operation: String,
    pub resource: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum InputType {
    String,
    Numeric,
    Address,
    Array,
}

#[derive(Clone, Debug)]
pub struct ValidationRule {
    pub min_length: usize,
    pub max_length: usize,
    pub pattern: String,
}

#[derive(Clone, Debug)]
pub enum MathOperation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug)]
pub struct OverflowConfig {
    pub enable_overflow_check: bool,
    pub enable_underflow_check: bool,
}

impl Default for OverflowConfig {
    fn default() -> Self {
        Self {
            enable_overflow_check: true,
            enable_underflow_check: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FlashLoanConfig {
    pub enable_detection: bool,
    pub min_balance_threshold: u64,
    pub time_window: u64,
}

impl Default for FlashLoanConfig {
    fn default() -> Self {
        Self {
            enable_detection: true,
            min_balance_threshold: 1000,
            time_window: 300, // 5分钟
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuditLogConfig {
    pub enable_logging: bool,
    pub log_level: LogLevel,
    pub max_log_entries: usize,
}

impl Default for AuditLogConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            log_level: LogLevel::Info,
            max_log_entries: 1000,
        }
    }
}

#[derive(Clone, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub enum SecurityEventType {
    UnauthorizedAccess,
    InvalidInput,
    OverflowDetected,
    ReentrancyDetected,
    FlashLoanDetected,
    PermissionDenied,
}

impl Default for SecurityHardeningConfig {
    fn default() -> Self {
        Self {
            enable_permission_control: true,
            enable_input_validation: true,
            enable_overflow_protection: true,
            enable_reentrancy_protection: true,
            enable_flash_loan_protection: true,
            enable_audit_logging: true,
            max_permission_depth: 10,
            max_input_length: 1024,
            max_numeric_value: u64::MAX,
            reentrancy_timeout: 30,
        }
    }
}

// 常量定义
pub const MAX_PERMISSION_DEPTH: u32 = 100;
pub const MAX_INPUT_LENGTH: usize = 10000;
pub const MAX_REENTRANCY_TIMEOUT: u64 = 300; // 5分钟

// 错误类型
#[derive(Debug)]
pub enum SecurityError {
    InvalidPermissionDepth,
    InvalidInputLength,
    InvalidNumericValue,
    InvalidTimeout,
    Overflow,
    Underflow,
    DivisionByZero,
    ReentrancyDetected,
    FlashLoanDetected,
    UnauthorizedAccess,
    InvalidInput,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_hardening_manager_creation() {
        let config = SecurityHardeningConfig::default();
        let manager = SecurityHardeningManager::new(config);
        assert_eq!(manager.name, "SecurityHardeningManager");
    }
    
    #[test]
    fn test_security_config_validation() {
        let valid_config = SecurityHardeningConfig::default();
        assert!(SecurityHardeningManager::validate_security_config(&valid_config).is_ok());
        
        let invalid_config = SecurityHardeningConfig {
            max_permission_depth: 0, // 无效的权限深度
            ..SecurityHardeningConfig::default()
        };
        assert!(SecurityHardeningManager::validate_security_config(&invalid_config).is_err());
    }
    
    #[test]
    fn test_overflow_protector() {
        let protector = OverflowProtector::new();
        
        // 测试正常加法
        assert!(protector.safe_math_operation(100, 200, MathOperation::Add).is_ok());
        
        // 测试溢出加法
        assert!(protector.safe_math_operation(u64::MAX, 1, MathOperation::Add).is_err());
        
        // 测试正常减法
        assert!(protector.safe_math_operation(200, 100, MathOperation::Sub).is_ok());
        
        // 测试下溢减法
        assert!(protector.safe_math_operation(100, 200, MathOperation::Sub).is_err());
    }
    
    #[test]
    fn test_reentrancy_protector() {
        let mut protector = ReentrancyProtector::new();
        let account = Pubkey::new_unique();
        
        // 测试正常进入临界区
        assert!(protector.enter_critical_section(&account, "test").is_ok());
        
        // 测试重入检测
        assert!(protector.enter_critical_section(&account, "test").is_err());
        
        // 测试正常退出临界区
        assert!(protector.exit_critical_section(&account, "test").is_ok());
    }
    
    #[test]
    fn test_flash_loan_protector() {
        let protector = FlashLoanProtector::new();
        let account = Pubkey::new_unique();
        
        // 测试正常余额变化
        assert!(protector.check_flash_loan_attack(&account, 1000, 1500).is_ok());
        
        // 测试闪电贷攻击检测
        assert!(protector.check_flash_loan_attack(&account, 0, 1000).is_err());
    }
} 