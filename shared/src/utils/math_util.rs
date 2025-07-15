use std::cmp::Ordering;

use crate::constants::{D18_U256, D9_U256, ONE_U256};
use crate::errors::ErrorCode::MathOverflow;
use anchor_lang::prelude::*;
use spl_math::uint::U256;

pub enum Rounding {
    Floor,
    Ceiling,
}

#[derive(Debug, Clone)]
pub struct Decimal(pub U256);

#[derive(Debug, Clone)]
pub struct TokenResult(pub u64);

pub trait IntoU256 {
    fn into_u256(self) -> U256;
}

impl IntoU256 for u64 {
    fn into_u256(self) -> U256 {
        U256::from(self)
    }
}

impl IntoU256 for u128 {
    fn into_u256(self) -> U256 {
        U256::from(self)
    }
}

impl IntoU256 for U256 {
    fn into_u256(self) -> U256 {
        self
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Decimal {}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Decimal {
    pub const ZERO: Self = Self(U256([0, 0, 0, 0]));
    pub const ONE: Self = Self(ONE_U256);
    pub const ONE_E18: Self = Self(D18_U256);
}

impl Decimal {
    pub fn from_plain(value: u64) -> Result<Self> {
        Ok(Decimal(
            U256::from(value)
                .checked_mul(D18_U256)
                .ok_or(MathOverflow)?,
        ))
    }

    pub fn from_token_amount<T: IntoU256>(value: T) -> Result<Self> {
        let result = value.into_u256().checked_mul(D9_U256).ok_or(MathOverflow)?;

        Ok(Decimal(result))
    }

    pub fn from_scaled<T: IntoU256>(value: T) -> Self {
        let result = value.into_u256();

        Decimal(result)
    }
}

impl Decimal {
    pub fn add(&self, other: &Self) -> Result<Self> {
        let result = self.0.checked_add(other.0).ok_or(MathOverflow)?;

        Ok(Decimal(result))
    }

    pub fn sub(&self, other: &Self) -> Result<Self> {
        let result = self.0.checked_sub(other.0).ok_or(MathOverflow)?;

        Ok(Decimal(result))
    }

    pub fn mul(&self, other: &Self) -> Result<Self> {
        let result = self.0.checked_mul(other.0).ok_or(MathOverflow)?;

        Ok(Decimal(result))
    }

    pub fn div(&self, other: &Self) -> Result<Self> {
        let result = self.0.checked_div(other.0).ok_or(MathOverflow)?;

        Ok(Decimal(result))
    }

    pub fn pow(&self, exponent: u64) -> Result<Self> {
        if exponent == 0 {
            return Ok(Decimal::ONE_E18);
        }

        if exponent == 1 {
            return Ok(self.clone());
        }

        let mut base = self.clone();
        let mut result = Decimal::ONE_E18;
        let mut exp = exponent;

        while exp > 0 {
            if exp & 1 == 1 {
                result = result.mul(&base)?.div(&Decimal::ONE_E18)?;
            }
            if exp > 1 {
                base = base.mul(&base)?.div(&Decimal::ONE_E18)?;
            }

            exp >>= 1;
        }

        Ok(result)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == U256::from(0)
    }
}

impl Decimal {
    const MAX_ITERATIONS: usize = 100;
    const EPSILON: U256 = U256([1, 0, 0, 0]);
    const NTH_ROOT_UPPER_BOUND: u64 = 1_000_000;
    const NTH_ROOT_MAX_ITERATIONS: usize = 15;

    pub const E: U256 = U256([2_718_281_828_459_045_235, 0, 0, 0]);

    pub fn nth_root(&self, n: u64) -> Result<Self> {
        if self.0 == Decimal::ZERO.0 {
            return Ok(Decimal::ZERO);
        }
        if self.0 == Decimal::ONE_E18.0 {
            return Ok(Decimal::ONE_E18);
        }

        if n > Self::NTH_ROOT_UPPER_BOUND {
            let x = Decimal::ONE_E18.sub(self)?;

            let n_decimal = Decimal::from_scaled(n);
            let first_term = x.div(&n_decimal)?;

            let x_squared = x.mul(&x)?;

            let n_value = n.checked_mul(n).ok_or(MathOverflow)?;

            let n_minus_one = n.checked_sub(1).ok_or(MathOverflow)?;

            let second_term = x_squared
                .mul(&Decimal::from_scaled(n_minus_one))?
                .div(&Decimal::ONE_E18)?
                .div(&Decimal::from_scaled(n_value))?
                .div(&Decimal::from_scaled(2u128))?;

            let x_cubed = x_squared.mul(&x)?;
            let n_minus_two = n.checked_sub(2).ok_or(MathOverflow)?;
            let numerator = n_minus_one.checked_mul(n_minus_two).ok_or(MathOverflow)?;

            let n_cubed = (n_value as u128)
                .checked_mul(n as u128)
                .ok_or(MathOverflow)?;

            let n_cubed_decimal = Decimal::from_scaled(n_cubed);

            let third_term = x_cubed // D54
                .mul(&Decimal::from_scaled(numerator))? // Multiply by (n-1)(n-2)
                .div(&Decimal::ONE_E18)? // Scale down to D54
                .div(&Decimal::ONE_E18)? // Scale down to D54
                .div(&n_cubed_decimal)? // Divide by n³
                .div(&Decimal::from_scaled(6u128))?; // Divide by 6

            let result = Decimal::ONE_E18
                .sub(&first_term)?
                .sub(&second_term)?
                .sub(&third_term)?;

            return Ok(result);
        }

        let mut low = Decimal::ZERO;
        let mut high = if self.0 > Decimal::ONE_E18.0 {
            self.clone() // D18
        } else {
            Decimal::ONE_E18 // D18
        };
        let target = self.clone(); // D18
        let two = &Decimal::from_scaled(2u128); // D18

        for _ in 0..Self::NTH_ROOT_MAX_ITERATIONS {
            let mid = low.add(&high)?.div(two)?; // D18
            let mut mid_pow = mid.clone(); // D18

            for _ in 1..n {
                mid_pow = mid_pow.mul(&mid)?.div(&Decimal::ONE_E18)?; // Keep at D18
            }

            match mid_pow.cmp(&target) {
                Ordering::Greater => high = mid,
                Ordering::Less => low = mid,
                Ordering::Equal => return Ok(mid),
            }
        }

        low.add(&high)?.div(two) // Final result in D18
    }

    pub fn ln(&self) -> Result<Option<Self>> {
        let one = Decimal::ONE_E18;

        let zero = Decimal(U256::from(0));

        if self.0 == one.0 {
            return Ok(Some(zero));
        }

        if self.0.is_zero() {
            return Ok(None);
        }

        let mut normalized = self.clone();
        let e = Decimal::from_scaled(Self::E);
        let mut power = 0i32;

        while normalized.0 < one.0 {
            // D18 x D18 = D36, so we need to div by D18
            normalized = normalized.mul(&e)?.div(&one)?;
            power -= 1;
        }

        while normalized.0 >= e.0 {
            // D18 / D18 = D0, so we need to mul by D18
            normalized = normalized.mul(&one)?.div(&e)?;
            power += 1;
        }

        let numerator = normalized.sub(&one)?;
        let denominator = normalized.add(&one)?;

        let z = numerator.mul(&one)?.div(&denominator)?;

        let z_squared = z.mul(&z)?.div(&one)?;

        let mut term = z.clone();
        let mut result = Decimal(U256::from(0));
        let mut n = 1u64;

        while n <= Self::MAX_ITERATIONS as u64 {
            result = result.add(&term.div(&Decimal::from_scaled(2 * n - 1))?)?;
            term = term.mul(&z_squared)?.div(&one)?;

            if term.0 < Self::EPSILON {
                break;
            }

            n += 1;
        }

        let mut final_result = result.mul(&Decimal::from_scaled(2u128))?;

        if power != 0 {
            let power_term = one.mul(&Decimal::from_scaled(power.unsigned_abs() as u64))?;

            if power > 0 {
                final_result = final_result.add(&power_term)?;
            } else {
                final_result = final_result.sub(&power_term)?;
            }
        }

        final_result = Decimal(final_result.0);

        Ok(Some(final_result))
    }

    pub fn exp(&self, negate_result: bool) -> Result<Option<Self>> {
        if self.0 == U256::from(0) {
            return Ok(Some(Decimal::ONE_E18));
        }

        let mut term = Decimal::ONE_E18;
        let mut result = term.clone();
        let mut n = 1u64;

        while n <= Self::MAX_ITERATIONS as u64 {
            term = term.mul(self)?.div(&Decimal::from_plain(n)?)?;
            result = result.add(&term)?;

            if term.0 < Self::EPSILON {
                break;
            }

            n += 1;
        }

        if negate_result {
            //e^(-x) = 1/(e^x)
            Ok(Some(Decimal(
                D18_U256
                    .checked_mul(D18_U256)
                    .ok_or(MathOverflow)?
                    .checked_div(result.0)
                    .ok_or(MathOverflow)?,
            )))
        } else {
            Ok(Some(result))
        }
    }
}
