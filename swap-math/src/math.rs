//! Math helpers

use num_traits::ToPrimitive;
use swap_client::fees::Fees;

const MAX: u64 = 1 << 32;
const MAX_BIG: u64 = 1 << 48;
const MAX_SMALL: u64 = 1 << 16;

/// Multiplies two u64s then divides by the third number.
/// This function attempts to use 64 bit math if possible.
#[inline(always)]
pub fn mul_div(a: u64, b: u64, c: u64) -> Option<u64> {
    if a > MAX || b > MAX {
        (a as u128)
            .checked_mul(b as u128)?
            .checked_div(c as u128)?
            .to_u64()
    } else {
        a.checked_mul(b)?.checked_div(c)
    }
}

/// Multiplies two u64s then divides by the third number.
/// This assumes that a > b.
#[inline(always)]
pub fn mul_div_imbalanced(a: u64, b: u64, c: u64) -> Option<u64> {
    if a > MAX_BIG || b > MAX_SMALL {
        (a as u128)
            .checked_mul(b as u128)?
            .checked_div(c as u128)?
            .to_u64()
    } else {
        a.checked_mul(b)?.checked_div(c)
    }
}

/// Calculates fees.
pub trait FeeCalculator {
    /// Applies the admin trade fee.
    fn admin_trade_fee(&self, fee_amount: u64) -> Option<u64>;
    /// Applies the admin withdraw fee.
    fn admin_withdraw_fee(&self, fee_amount: u64) -> Option<u64>;
    /// Applies the trade fee.
    fn trade_fee(&self, trade_amount: u64) -> Option<u64>;
    /// Applies the withdraw fee.
    fn withdraw_fee(&self, withdraw_amount: u64) -> Option<u64>;
    /// Applies the normalized trade fee.
    fn normalized_trade_fee(&self, n_coins: u8, amount: u64) -> Option<u64>;
}

impl FeeCalculator for Fees {
    /// Apply admin trade fee
    fn admin_trade_fee(&self, fee_amount: u64) -> Option<u64> {
        mul_div_imbalanced(
            fee_amount,
            self.admin_trade_fee_numerator,
            self.admin_trade_fee_denominator,
        )
    }

    /// Apply admin withdraw fee
    fn admin_withdraw_fee(&self, fee_amount: u64) -> Option<u64> {
        mul_div_imbalanced(
            fee_amount,
            self.admin_withdraw_fee_numerator,
            self.admin_withdraw_fee_denominator,
        )
    }

    /// Compute trade fee from amount
    fn trade_fee(&self, trade_amount: u64) -> Option<u64> {
        mul_div_imbalanced(
            trade_amount,
            self.trade_fee_numerator,
            self.trade_fee_denominator,
        )
    }

    /// Compute withdraw fee from amount
    fn withdraw_fee(&self, withdraw_amount: u64) -> Option<u64> {
        mul_div_imbalanced(
            withdraw_amount,
            self.withdraw_fee_numerator,
            self.withdraw_fee_denominator,
        )
    }

    /// Compute normalized fee for symmetric/asymmetric deposits/withdraws
    fn normalized_trade_fee(&self, n_coins: u8, amount: u64) -> Option<u64> {
        // adjusted_fee_numerator: uint256 = self.fee * N_COINS / (4 * (N_COINS - 1))
        // The number 4 comes from Curve, originating from some sort of calculus
        // https://github.com/curvefi/curve-contract/blob/e5fb8c0e0bcd2fe2e03634135806c0f36b245511/tests/simulation.py#L124
        let adjusted_trade_fee_numerator = mul_div(
            self.trade_fee_numerator,
            n_coins.into(),
            (n_coins.checked_sub(1)?).checked_mul(4)?.into(),
        )?;

        mul_div(
            amount,
            adjusted_trade_fee_numerator,
            self.trade_fee_denominator,
        )
    }
}
