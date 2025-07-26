//!
//! linear_algebra.rs - 线性代数函数实现
//!
//! 本文件实现LinearAlgebra结构体及其所有线性代数方法，严格遵循Rust、Anchor、SOLID最佳实践，
//! 并逐行专业注释，便于审计、维护、扩展。

use crate::error::StrategyError;
use anchor_lang::prelude::*;

/// 线性代数工具结构体
/// - 提供常用线性代数函数实现
pub struct LinearAlgebra;

impl LinearAlgebra {
    /// 矩阵乘法
    pub fn matrix_multiply(a: &[Vec<Decimal>], b: &[Vec<Decimal>]) -> Result<Vec<Vec<Decimal>>> {
        let n = a.len();
        let m = b[0].len();
        let p = b.len();
        if a[0].len() != p {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let mut result = vec![vec![Decimal::ZERO; m]; n];
        for i in 0..n {
            for j in 0..m {
                for k in 0..p {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        Ok(result)
    }

    /// 2x2矩阵行列式
    pub fn determinant(matrix: &[Vec<Decimal>]) -> Result<Decimal> {
        if matrix.len() != 2 || matrix[0].len() != 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        Ok(matrix[0][0] * matrix[1][1] - matrix[0][1] * matrix[1][0])
    }

    /// 2x2矩阵求逆
    pub fn inverse_2x2(matrix: &[Vec<Decimal>]) -> Result<Vec<Vec<Decimal>>> {
        let det = Self::determinant(matrix)?;
        if det == Decimal::ZERO {
            return Err(StrategyError::MathOverflow);
        }
        let inv = vec![
            vec![matrix[1][1] / det, -matrix[0][1] / det],
            vec![-matrix[1][0] / det, matrix[0][0] / det],
        ];
        Ok(inv)
    }

    /// 2x2矩阵特征值
    pub fn eigenvalues_2x2(matrix: &[Vec<Decimal>]) -> Result<(Decimal, Decimal)> {
        if matrix.len() != 2 || matrix[0].len() != 2 {
            return Err(StrategyError::InvalidStrategyParameters);
        }
        let a = matrix[0][0];
        let b = matrix[0][1];
        let c = matrix[1][0];
        let d = matrix[1][1];
        let trace = a + d;
        let det = a * d - b * c;
        let discrim = (trace * trace - Decimal::from(4u64) * det).sqrt();
        Ok(((trace + discrim) / Decimal::from(2u64), (trace - discrim) / Decimal::from(2u64)))
    }
} 