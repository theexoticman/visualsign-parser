# Jupiter Swap Decoder Integration

## Overview
Successfully integrated Jupiter swap decoding into the visualsign-solana crate to decode Jupiter Swap instructions when the program ID is `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`.

## Features Implemented

### 1. Jupiter Swap Instruction Types
- **Route**: Basic swap with input amount, quoted output amount, and slippage
- **ExactOutRoute**: Exact output swap with max input amount and exact output amount  
- **SharedAccountsRoute**: Shared accounts route with input/output amounts and slippage
- **Unknown**: Fallback for unrecognized instruction types

### 2. Instruction Parsing
- Discriminator-based parsing for different Jupiter instruction types
- Route instruction discriminator: `[0x2a, 0xad, 0xe3, 0x7a, 0x97, 0xcb, 0x17, 0xe5]`
- ExactOutRoute discriminator: `[0x2a, 0xb6, 0xd0, 0x0c, 0xa8, 0xdf, 0xd7, 0x4b]`
- SharedAccountsRoute discriminator: `[0x2a, 0xd4, 0xb6, 0x2f, 0xae, 0xaa, 0xf2, 0x3a]`

### 3. Visual Sign Integration
- **Condensed View**: Shows formatted instruction summary
- **Expanded View**: Detailed breakdown with typed fields:
  - Program ID (TextV2)
  - Input Amount (AmountV2) 
  - Quoted Output Amount (AmountV2)
  - Slippage (Number in basis points)
  - Raw Data (TextV2)

### 4. Formatting
- Human-readable instruction descriptions
- Proper field types for UI rendering
- Amount formatting with token units
- Slippage display in basis points

## Code Changes

### Dependencies Added
No external dependencies needed - implemented as a self-contained parser.

### Files Modified
- `src/lib.rs`: 
  - Added `JupiterSwapInstruction` enum
  - Added parsing functions for Jupiter instructions
  - Added formatting functions for user display
  - Added expanded field creation for detailed views
  - Integrated into main instruction matching logic

### Key Functions Added
- `parse_jupiter_swap_instruction()`: Parses raw instruction data
- `format_jupiter_swap_instruction()`: Formats for user display
- `create_jupiter_swap_expanded_fields()`: Creates detailed field breakdown
- Individual parsers for Route, ExactOutRoute, and SharedAccountsRoute

## Testing

### Example Usage
A complete example is provided in `examples/jupiter_example.rs` that demonstrates:
- Creating a mock Jupiter swap transaction
- Converting to VisualSign payload
- Extracting and displaying the parsed instruction details

### Test Output
```
✅ Successfully decoded Jupiter swap transaction!
🔄 Found Jupiter instruction: Jupiter Swap: 1000000 -> 2000000 (slippage: 50bps)
   Expanded fields: 5
     0: Program ID (TextV2)
     1: Input Amount (AmountV2)  
     2: Quoted Output Amount (AmountV2)
     3: Slippage (Number)
     4: Raw Data (TextV2)
```

## Usage

When a Solana transaction contains a Jupiter swap instruction with program ID `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`, the visualsign-solana crate will automatically:

1. Detect the Jupiter program ID
2. Parse the instruction data based on discriminators
3. Extract swap parameters (amounts, slippage, etc.)
4. Format for display in both condensed and expanded views
5. Use appropriate field types for proper UI rendering

The integration is seamless and requires no additional configuration - Jupiter swaps will be automatically decoded and formatted when present in transactions.
