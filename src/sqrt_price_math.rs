use std::ops::Shl;

use ethers::types::{I256, U256};

use crate::{
    error::UniswapV3Error,
    full_math::{mul_div, mul_div_rounding_up},
    unsafe_math::div_rounding_up,
};

// returns (sqrtQX96)
pub fn get_next_sqrt_price_from_input(
    sqrt_price: U256,
    liquidity: u128,
    amount_in: U256,
    zero_for_one: bool,
) -> Result<U256, UniswapV3Error> {
    if sqrt_price == U256::zero() {
        return Err(UniswapV3Error::SqrtPriceIsZero());
    } else if liquidity == 0 {
        return Err(UniswapV3Error::LiquidityIsZero());
    }

    if zero_for_one {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price, liquidity, amount_in, true)
    } else {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price, liquidity, amount_in, true)
    }
}

// returns (sqrtQX96)
pub fn get_next_sqrt_price_from_output(
    sqrt_price: U256,
    liquidity: u128,
    amount_out: U256,
    zero_for_one: bool,
) -> Result<U256, UniswapV3Error> {
    if sqrt_price == U256::zero() {
        return Err(UniswapV3Error::SqrtPriceIsZero());
    } else if liquidity == 0 {
        return Err(UniswapV3Error::LiquidityIsZero());
    }

    if zero_for_one {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price, liquidity, amount_out, false)
    } else {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price, liquidity, amount_out, false)
    }
}

// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_from_amount_0_rounding_up(
    sqrt_price_x_96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U256, UniswapV3Error> {
    if amount.is_zero() {
        return Ok(sqrt_price_x_96);
    }

    let numerator_1 = U256::from(liquidity).shl(96);

    if add {
        let product = amount * sqrt_price_x_96;

        if product / amount == sqrt_price_x_96 {
            let denominator = numerator_1 + product;

            if denominator >= numerator_1 {
                return mul_div_rounding_up(numerator_1, sqrt_price_x_96, denominator);
            }
        }

        Ok(div_rounding_up(
            numerator_1,
            (numerator_1 / sqrt_price_x_96) + amount,
        ))
    } else {
        let product = amount * sqrt_price_x_96;
        if product / amount == sqrt_price_x_96 && (numerator_1 > product) {
            let denominator = numerator_1 - product;

            mul_div_rounding_up(numerator_1, sqrt_price_x_96, denominator)
        } else {
            Err(UniswapV3Error::ProductDivAmount())
        }
    }
}

// returns (uint160 sqrtQX96)
pub fn get_next_sqrt_price_from_amount_1_rounding_down(
    sqrt_price_x_96: U256,
    liquidity: u128,
    amount: U256,
    add: bool,
) -> Result<U256, UniswapV3Error> {
    if add {
        let quotent = if amount <= U256::from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF") {
            amount.shl(96) / liquidity
        } else {
            mul_div(
                amount,
                U256::from("0x1000000000000000000000000"),
                U256::from(liquidity),
            )?
        };

        Ok(sqrt_price_x_96 + quotent)
    } else {
        let quotent = if amount <= U256::from("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF") {
            div_rounding_up(amount.shl(96), U256::from(liquidity))
        } else {
            mul_div_rounding_up(
                amount,
                U256::from("0x1000000000000000000000000"),
                U256::from(liquidity),
            )?
        };

        Ok(sqrt_price_x_96 - quotent)
    }
}

// returns (uint256 amount0)
pub fn _get_amount_0_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: i128,
    round_up: bool,
) -> Result<U256, UniswapV3Error> {
    let (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96) = if sqrt_ratio_a_x_96 > sqrt_ratio_b_x_96 {
        (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96)
    } else {
        (sqrt_ratio_b_x_96, sqrt_ratio_a_x_96)
    };

    let numerator_1 = U256::from(liquidity).shl(96);
    let numerator_2 = sqrt_ratio_a_x_96 - sqrt_ratio_b_x_96;

    if sqrt_ratio_a_x_96 == U256::zero() {
        return Err(UniswapV3Error::SqrtPriceIsZero());
    }

    if round_up {
        let numerator_partial = mul_div_rounding_up(numerator_1, numerator_2, sqrt_ratio_b_x_96)?;
        Ok(div_rounding_up(numerator_partial, sqrt_ratio_a_x_96))
    } else {
        Ok(mul_div(numerator_1, numerator_2, sqrt_ratio_b_x_96)? / sqrt_ratio_a_x_96)
    }
}

// returns (uint256 amount1)
pub fn _get_amount_1_delta(
    mut sqrt_ratio_a_x_96: U256,
    mut sqrt_ratio_b_x_96: U256,
    liquidity: i128,
    round_up: bool,
) -> Result<U256, UniswapV3Error> {
    (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96) = if sqrt_ratio_a_x_96 > sqrt_ratio_b_x_96 {
        (sqrt_ratio_a_x_96, sqrt_ratio_b_x_96)
    } else {
        (sqrt_ratio_b_x_96, sqrt_ratio_a_x_96)
    };

    if round_up {
        mul_div_rounding_up(
            U256::from(liquidity),
            sqrt_ratio_b_x_96 - sqrt_ratio_a_x_96,
            U256::from("0x1000000000000000000000000"),
        )
    } else {
        mul_div(
            U256::from(liquidity),
            sqrt_ratio_b_x_96 - sqrt_ratio_a_x_96,
            U256::from("0x1000000000000000000000000"),
        )
    }
}

pub fn get_amount_0_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: i128,
) -> Result<I256, UniswapV3Error> {
    if liquidity < 0 {
        Ok(I256::from_raw(_get_amount_0_delta(
            sqrt_ratio_b_x_96,
            sqrt_ratio_a_x_96,
            -liquidity as i128,
            false,
        )?))
    } else {
        Ok(I256::from_raw(_get_amount_0_delta(
            sqrt_ratio_a_x_96,
            sqrt_ratio_b_x_96,
            liquidity,
            true,
        )?))
    }
}

pub fn get_amount_1_delta(
    sqrt_ratio_a_x_96: U256,
    sqrt_ratio_b_x_96: U256,
    liquidity: i128,
) -> Result<I256, UniswapV3Error> {
    if liquidity < 0 {
        Ok(I256::from_raw(_get_amount_1_delta(
            sqrt_ratio_b_x_96,
            sqrt_ratio_a_x_96,
            -liquidity as i128,
            false,
        )?))
    } else {
        Ok(I256::from_raw(_get_amount_1_delta(
            sqrt_ratio_a_x_96,
            sqrt_ratio_b_x_96,
            liquidity,
            true,
        )?))
    }
}
