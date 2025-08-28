#![allow(dead_code)]

crate::chain_config! {
  config CETUS_CONFIG as Config;

  cetus_mainnet => {
      package_id => 0xb2db7142fa83210a7d78d9c12ac49c043b3cbbd482224fea6e3da00aa5a5ae2d,
      modules: {
        pool_script_v2 => PoolScriptV2Functions: {
          swap_a2b as SwapA2B => SwapA2BIndexes(
                by_amount_in as ByAmountIn: bool => 4 => get_by_amount_in,
                amount as Amount: u64 => 5 => get_amount,
                amount_limit as AmountLimit: u64 => 6 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 7 => get_sqrt_price_limit,
          ),
          swap_b2a as SwapB2A => SwapB2AIndexes(
                by_amount_in as ByAmountIn: bool => 4 => get_by_amount_in,
                amount as Amount: u64 => 5 => get_amount,
                amount_limit as AmountLimit: u64 => 6 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 7 => get_sqrt_price_limit,
          ),
        },
      }
  },
}
