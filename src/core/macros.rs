/*!
 * Core Macros Module
 *
 * Provides utility macros for common operations.
 */

/// Safe math macro for checked arithmetic operations
#[macro_export]
macro_rules! safe_add {
    ($a:expr, $b:expr) => {
        $a.checked_add($b)
            .ok_or($crate::error::StrategyError::MathOverflow)?
    };
}

#[macro_export]
macro_rules! safe_sub {
    ($a:expr, $b:expr) => {
        $a.checked_sub($b)
            .ok_or($crate::error::StrategyError::MathOverflow)?
    };
}

#[macro_export]
macro_rules! safe_mul {
    ($a:expr, $b:expr) => {
        $a.checked_mul($b)
            .ok_or($crate::error::StrategyError::MathOverflow)?
    };
}

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

/// Version check macro
#[macro_export]
macro_rules! version_check {
    ($account:expr, $min_version:expr) => {
        if $account.version < $min_version {
            return Err($crate::error::StrategyError::IncompatibleVersion.into());
        }
    };
}

/// Risk assessment macro
#[macro_export]
macro_rules! assess_risk {
    ($value:expr, $limit:expr, $error:expr) => {
        if $value > $limit {
            return Err($error.into());
        }
    };
}

/// Batch processing macro
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

/// Auto migration macro
#[macro_export]
macro_rules! auto_migrate {
    ($account:expr, $current_version:expr) => {
        if $account.version < $current_version {
            $account.migrate_to_version($current_version)?;
        }
    };
}
