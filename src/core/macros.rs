/*!
 * 核心宏模块
 *
 * 提供常用操作的实用宏定义。
 *
 * # 设计说明
 * - 所有宏均为安全、可复用、便于审计的最小功能单元
 * - 错误处理严格，返回统一的错误类型，便于 Anchor/Solana 程序集成
 * - 适用于链上高安全性、可维护性场景
 */

/// 安全加法宏
///
/// # 用途
/// - 用于溢出保护的加法运算，失败时返回MathOverflow错误
///
/// # 参数
/// - $a: 左操作数（实现了checked_add的类型）
/// - $b: 右操作数
///
/// # 返回
/// - 加法结果（溢出时返回StrategyError::MathOverflow）
#[macro_export]
macro_rules! safe_add {
    ($a:expr, $b:expr) => {
        $a.checked_add($b)
            .ok_or($crate::error::StrategyError::MathOverflow)?
    };
}

/// 安全减法宏
///
/// # 用途
/// - 用于下溢保护的减法运算，失败时返回MathOverflow错误
///
/// # 参数
/// - $a: 被减数（实现了checked_sub的类型）
/// - $b: 减数
///
/// # 返回
/// - 减法结果（下溢时返回StrategyError::MathOverflow）
#[macro_export]
macro_rules! safe_sub {
    ($a:expr, $b:expr) => {
        $a.checked_sub($b)
            .ok_or($crate::error::StrategyError::MathOverflow)?
    };
}

/// 安全乘法宏
///
/// # 用途
/// - 用于溢出保护的乘法运算，失败时返回MathOverflow错误
///
/// # 参数
/// - $a: 左操作数（实现了checked_mul的类型）
/// - $b: 右操作数
///
/// # 返回
/// - 乘法结果（溢出时返回StrategyError::MathOverflow）
#[macro_export]
macro_rules! safe_mul {
    ($a:expr, $b:expr) => {
        $a.checked_mul($b)
            .ok_or($crate::error::StrategyError::MathOverflow)?
    };
}

/// 安全除法宏
///
/// # 用途
/// - 用于零检查和溢出保护的除法运算，除数为0时返回DivisionByZero错误，否则溢出返回MathOverflow错误
///
/// # 参数
/// - $a: 被除数（实现了checked_div的类型）
/// - $b: 除数
///
/// # 返回
/// - 除法结果（除数为0时返回StrategyError::DivisionByZero，溢出时返回StrategyError::MathOverflow）
#[macro_export]
macro_rules! safe_div {
    ($a:expr, $b:expr) => {
        if $b == 0 {
            return Err($crate::error::StrategyError::DivisionByZero.into());
        }
        $a.checked_div($b)
            .ok_or($crate::error::StrategyError::MathOverflow)?
    };
}

/// 版本检查宏
///
/// # 用途
/// - 用于账户结构体的版本兼容性校验，不满足最小版本要求时返回IncompatibleVersion错误
///
/// # 参数
/// - $account: 账户结构体（需含version字段）
/// - $min_version: 最小兼容版本
///
/// # 返回
/// - 校验失败时返回StrategyError::IncompatibleVersion
#[macro_export]
macro_rules! version_check {
    ($account:expr, $min_version:expr) => {
        if $account.version < $min_version {
            return Err($crate::error::StrategyError::IncompatibleVersion.into());
        }
    };
}

/// 风险评估宏
///
/// # 用途
/// - 用于统一的风险阈值判断，超限时返回指定错误
///
/// # 参数
/// - $value: 被评估值
/// - $limit: 阈值
/// - $error: 错误类型（需实现Into<ProgramError>）
///
/// # 返回
/// - 超限时返回$error
#[macro_export]
macro_rules! assess_risk {
    ($value:expr, $limit:expr, $error:expr) => {
        if $value > $limit {
            return Err($error.into());
        }
    };
}

/// 批量处理宏
///
/// # 用途
/// - 用于将大批量数据分批处理，支持自定义处理器和批大小，便于算力优化
///
/// # 参数
/// - $items: 数据集合（实现chunks方法的集合）
/// - $processor: 处理函数（Fn(&T) -> anchor_lang::Result<U>）
/// - $batch_size: 每批处理数量
///
/// # 返回
/// - 处理结果集合或错误
#[macro_export]
macro_rules! process_batch {
    ($items:expr, $processor:expr, $batch_size:expr) => {{
        let mut results = Vec::new();
        for chunk in $items.chunks($batch_size) {
            for item in chunk {
                results.push($processor(item)?);
            }
        }
        results
    }};
}

/// 自动迁移宏
///
/// # 用途
/// - 用于账户结构体的版本自动迁移，若当前版本小于目标版本则自动调用迁移方法
///
/// # 参数
/// - $account: 账户结构体（需含version字段和migrate_to_version方法）
/// - $current_version: 目标版本
///
/// # 返回
/// - 迁移失败时返回迁移方法的错误
#[macro_export]
macro_rules! auto_migrate {
    ($account:expr, $current_version:expr) => {
        if $account.version < $current_version {
            $account.migrate_to_version($current_version)?;
        }
    };
}
