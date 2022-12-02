use std::collections::HashMap;

use ethers::types::U256;

pub struct Tick {
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
    pub fee_growth_outside_0_x_128: U256,
    pub fee_growth_outside_1_x_128: U256,
    pub tick_cumulative_outside: U256,
    pub seconds_per_liquidity_outside_x_128: U256,
    pub seconds_outside: u32,
    pub initialized: bool,
}

fn cross(tick_mapping: HashMap<i32, Tick>, tick: i32) -> i128 {
    if let Some(tick) = tick_mapping.get(&tick) {
        tick.liquidity_net
    } else {
        0
    }
}
