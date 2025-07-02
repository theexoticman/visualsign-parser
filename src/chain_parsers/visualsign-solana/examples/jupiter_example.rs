//! Example demonstrating Jupiter swap decoding in visualsign-solana
//!
//! This example shows how the Jupiter swap decoder integrates with the
//! visualsign-solana crate to decode Jupiter swap instructions.

use solana_sdk::{
    instruction::Instruction, message::Message, pubkey::Pubkey, transaction::Transaction,
};
use std::str::FromStr;
use visualsign::vsptrait::VisualSignOptions;

fn main() {
    println!("Jupiter Swap Decoder Integration Example");
    println!("========================================");

    // Create a mock Jupiter swap transaction
    let jupiter_program_id =
        Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap();

    // Mock Jupiter Route instruction data
    let instruction_data = vec![
        0x2a, 0xad, 0xe3, 0x7a, 0x97, 0xcb, 0x17, 0xe5, // Route discriminator
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // route plan (simplified)
        0x40, 0x42, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, // in_amount: 1000000
        0x80, 0x84, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, // quoted_out_amount: 2000000
    ];

    let accounts = vec![
        solana_sdk::instruction::AccountMeta::new_readonly(Pubkey::new_unique(), false),
        solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), true),
        solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), false),
        solana_sdk::instruction::AccountMeta::new(Pubkey::new_unique(), false),
    ];

    let instruction = Instruction {
        program_id: jupiter_program_id,
        accounts,
        data: instruction_data,
    };

    let payer = Pubkey::new_unique();
    let message = Message::new(&[instruction], Some(&payer));
    let transaction = Transaction::new_unsigned(message);

    // Convert to visual sign payload
    let options = VisualSignOptions {
        decode_transfers: false,
        transaction_name: Some("Jupiter Swap Example".to_string()),
    };

    match visualsign_solana::transaction_to_visual_sign(transaction, options) {
        Ok(payload) => {
            println!("✅ Successfully decoded Jupiter swap transaction!");
            println!("Transaction title: {}", payload.title);
            println!("Number of fields: {}", payload.fields.len());

            // Look for Jupiter instruction
            for (i, field) in payload.fields.iter().enumerate() {
                if let visualsign::SignablePayloadField::PreviewLayout {
                    preview_layout,
                    common: _,
                } = field
                {
                    if let Some(title) = &preview_layout.title {
                        if title.text.contains("Jupiter") {
                            println!(
                                "🔄 Found Jupiter instruction at field {}: {}",
                                i, title.text
                            );

                            if let Some(expanded) = &preview_layout.expanded {
                                println!("   Expanded fields: {}", expanded.fields.len());
                                for (j, exp_field) in expanded.fields.iter().enumerate() {
                                    match &exp_field.signable_payload_field {
                                        visualsign::SignablePayloadField::TextV2 {
                                            common, ..
                                        } => {
                                            println!("     {}: {} (TextV2)", j, common.label);
                                        }
                                        visualsign::SignablePayloadField::AmountV2 {
                                            common,
                                            ..
                                        } => {
                                            println!("     {}: {} (AmountV2)", j, common.label);
                                        }
                                        visualsign::SignablePayloadField::Number {
                                            common, ..
                                        } => {
                                            println!("     {}: {} (Number)", j, common.label);
                                        }
                                        _ => {
                                            println!("     {}: Other field type", j);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to decode transaction: {:?}", e);
        }
    }
}
