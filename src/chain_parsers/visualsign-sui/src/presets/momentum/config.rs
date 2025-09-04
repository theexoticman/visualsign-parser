#![allow(dead_code)]

crate::chain_config! {
  config MOMENTUM_CONFIG as Config;

  momentum_mainnet => {
      package_id => 0xcf60a40f45d46fc1e828871a647c1e25a0915dec860d2662eb10fdb382c3c1d1,
      modules as MomentumModules: {
        collect as Collect => CollectFunctions: {
          fee as Fee => FeeIndexes(),
          reward as Reward => RewardIndexes(),
        },
        liquidity as Liquidity => LiquidityFunctions: {
          remove_liquidity as RemoveLiquidity => RemoveLiquidityIndexes(),
          close_position as ClosePosition => ClosePositionIndexes(),
          add_liquidity as AddLiquidity => AddLiquidityIndexes(),
          open_position as OpenPosition => OpenPositionIndexes(),
        },
        position as Position => PositionFunctions: {
          liquidity as Liquidity => PositionLiquidityIndexes(),
        },
        trade as Trade => TradeFunctions: {
          flash_swap as FlashSwap => FlashSwapIndexes(),
          repay_flash_swap as RepayFlashSwap => RepayFlashSwapIndexes(),
          swap_receipt_debts as SwapReceiptDebts => SwapReceiptDebtsIndexes(),
        },
      }
  },
}
