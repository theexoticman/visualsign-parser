//! Jupiter swap preset implementation for Solana

mod config;

use crate::core::{
    InstructionVisualizer, SolanaIntegrationConfig, VisualizerContext, VisualizerKind,
};
use crate::utils::{SwapTokenInfo, get_token_info};
use config::JupiterSwapConfig;
use visualsign::errors::VisualSignError;
use visualsign::field_builders::{
    create_amount_field, create_number_field, create_raw_data_field, create_text_field,
};
use visualsign::{
    AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldCommon,
    SignablePayloadFieldListLayout, SignablePayloadFieldPreviewLayout, SignablePayloadFieldTextV2,
};

// Jupiter instruction discriminators (8-byte values)
const JUPITER_ROUTE_DISCRIMINATOR: [u8; 8] = [0xe5, 0x17, 0xcb, 0x97, 0x7a, 0xe3, 0xad, 0x2a];
const JUPITER_EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] =
    [0x4b, 0xd7, 0xdf, 0xa8, 0x0c, 0xd0, 0xb6, 0x2a];
const JUPITER_SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR: [u8; 8] =
    [0x3a, 0xf2, 0xaa, 0xae, 0x2f, 0xb6, 0xd4, 0x2a];

#[derive(Debug, Clone)]
pub enum JupiterSwapInstruction {
    Route {
        in_token: Option<SwapTokenInfo>,
        out_token: Option<SwapTokenInfo>,
        slippage_bps: u16,
    },
    ExactOutRoute {
        in_token: Option<SwapTokenInfo>,
        out_token: Option<SwapTokenInfo>,
        slippage_bps: u16,
    },
    SharedAccountsRoute {
        in_token: Option<SwapTokenInfo>,
        out_token: Option<SwapTokenInfo>,
        slippage_bps: u16,
    },
    Unknown,
}

impl JupiterSwapInstruction {
    /// Parse amounts from the last 16 bytes of instruction data
    fn parse_amounts_from_data(data: &[u8]) -> Result<(u64, u64), &'static str> {
        if data.len() < 16 {
            return Err("Instruction data too short");
        }

        let len = data.len();
        let in_amount = u64::from_le_bytes([
            data[len - 16],
            data[len - 15],
            data[len - 14],
            data[len - 13],
            data[len - 12],
            data[len - 11],
            data[len - 10],
            data[len - 9],
        ]);
        let out_amount = u64::from_le_bytes([
            data[len - 8],
            data[len - 7],
            data[len - 6],
            data[len - 5],
            data[len - 4],
            data[len - 3],
            data[len - 2],
            data[len - 1],
        ]);

        Ok((in_amount, out_amount))
    }
}

// Create a static instance that we can reference
static JUPITER_CONFIG: JupiterSwapConfig = JupiterSwapConfig;

pub struct JupiterSwapVisualizer;

impl InstructionVisualizer for JupiterSwapVisualizer {
    fn visualize_tx_commands(
        &self,
        context: &VisualizerContext,
    ) -> Result<AnnotatedPayloadField, VisualSignError> {
        let instruction = context
            .current_instruction()
            .ok_or_else(|| VisualSignError::MissingData("No instruction found".into()))?;

        let instruction_accounts: Vec<String> = instruction
            .accounts
            .iter()
            .map(|account| account.pubkey.to_string())
            .collect();

        let jupiter_instruction =
            parse_jupiter_swap_instruction(&instruction.data, &instruction_accounts)
                .map_err(|e| VisualSignError::DecodeError(e.to_string()))?;

        let instruction_text = format_jupiter_swap_instruction(&jupiter_instruction);

        let condensed = SignablePayloadFieldListLayout {
            fields: vec![
                create_text_field("Instruction", &instruction_text)
                    .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
            ],
        };

        let expanded = SignablePayloadFieldListLayout {
            fields: create_jupiter_swap_expanded_fields(
                &jupiter_instruction,
                &instruction.program_id.to_string(),
                &instruction.data,
            )?,
        };

        let preview_layout = SignablePayloadFieldPreviewLayout {
            title: Some(SignablePayloadFieldTextV2 {
                text: instruction_text.clone(),
            }),
            subtitle: Some(SignablePayloadFieldTextV2 {
                text: String::new(),
            }),
            condensed: Some(condensed),
            expanded: Some(expanded),
        };

        let fallback_text = format!(
            "Program ID: {}\nData: {}",
            instruction.program_id,
            hex::encode(&instruction.data)
        );

        Ok(AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::PreviewLayout {
                common: SignablePayloadFieldCommon {
                    label: format!("Instruction {}", context.instruction_index() + 1),
                    fallback_text,
                },
                preview_layout,
            },
        })
    }

    fn get_config(&self) -> Option<&dyn SolanaIntegrationConfig> {
        Some(&JUPITER_CONFIG)
    }

    fn kind(&self) -> VisualizerKind {
        VisualizerKind::Dex("Jupiter")
    }
}

fn parse_jupiter_swap_instruction(
    data: &[u8],
    accounts: &[String],
) -> Result<JupiterSwapInstruction, &'static str> {
    if data.len() < 8 {
        return Err("Invalid instruction data length");
    }

    let discriminator = &data[0..8];

    match discriminator {
        d if d == &JUPITER_ROUTE_DISCRIMINATOR => parse_route_instruction(data, accounts),
        d if d == &JUPITER_EXACT_OUT_ROUTE_DISCRIMINATOR => {
            parse_exact_out_route_instruction(data, accounts)
        }
        d if d == &JUPITER_SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR => {
            parse_shared_accounts_route_instruction(data, accounts)
        }
        _ => Ok(JupiterSwapInstruction::Unknown),
    }
}

fn parse_route_instruction(
    data: &[u8],
    accounts: &[String],
) -> Result<JupiterSwapInstruction, &'static str> {
    let (in_amount, out_amount) = JupiterSwapInstruction::parse_amounts_from_data(data)?;

    let in_token = accounts.get(0).map(|addr| get_token_info(addr, in_amount));
    let out_token = accounts.get(1).map(|addr| get_token_info(addr, out_amount));

    Ok(JupiterSwapInstruction::Route {
        in_token,
        out_token,
        slippage_bps: 50, // Default slippage
    })
}

fn parse_exact_out_route_instruction(
    data: &[u8],
    accounts: &[String],
) -> Result<JupiterSwapInstruction, &'static str> {
    let (in_amount, out_amount) = JupiterSwapInstruction::parse_amounts_from_data(data)?;

    let in_token = accounts.get(0).map(|addr| get_token_info(addr, in_amount));
    let out_token = accounts.get(1).map(|addr| get_token_info(addr, out_amount));

    Ok(JupiterSwapInstruction::ExactOutRoute {
        in_token,
        out_token,
        slippage_bps: 50, // Default slippage
    })
}

fn parse_shared_accounts_route_instruction(
    data: &[u8],
    accounts: &[String],
) -> Result<JupiterSwapInstruction, &'static str> {
    let (in_amount, out_amount) = JupiterSwapInstruction::parse_amounts_from_data(data)?;

    let in_token = accounts.get(0).map(|addr| get_token_info(addr, in_amount));
    let out_token = accounts.get(1).map(|addr| get_token_info(addr, out_amount));

    Ok(JupiterSwapInstruction::SharedAccountsRoute {
        in_token,
        out_token,
        slippage_bps: 50, // Default slippage
    })
}

fn format_jupiter_swap_instruction(instruction: &JupiterSwapInstruction) -> String {
    match instruction {
        JupiterSwapInstruction::Route {
            in_token,
            out_token,
            slippage_bps,
        }
        | JupiterSwapInstruction::ExactOutRoute {
            in_token,
            out_token,
            slippage_bps,
        }
        | JupiterSwapInstruction::SharedAccountsRoute {
            in_token,
            out_token,
            slippage_bps,
        } => {
            let instruction_type = match instruction {
                JupiterSwapInstruction::Route { .. } => "Jupiter Swap",
                JupiterSwapInstruction::ExactOutRoute { .. } => "Jupiter Exact Out Route",
                JupiterSwapInstruction::SharedAccountsRoute { .. } => {
                    "Jupiter Shared Accounts Route"
                }
                _ => unreachable!(),
            };

            format!(
                "{}: {} {} → {} {} (slippage: {}bps)",
                instruction_type,
                format_token_amount(in_token),
                format_token_symbol(in_token),
                format_token_amount(out_token),
                format_token_symbol(out_token),
                slippage_bps
            )
        }
        JupiterSwapInstruction::Unknown => "Jupiter: Unknown Instruction".to_string(),
    }
}

fn format_token_amount(token: &Option<SwapTokenInfo>) -> String {
    token
        .as_ref()
        .map(|t| t.amount.to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn format_token_symbol(token: &Option<SwapTokenInfo>) -> String {
    token
        .as_ref()
        .map(|t| t.symbol.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn create_jupiter_swap_expanded_fields(
    instruction: &JupiterSwapInstruction,
    program_id: &str,
    data: &[u8],
) -> Result<Vec<AnnotatedPayloadField>, VisualSignError> {
    let mut fields = vec![
        create_text_field("Program ID", program_id)
            .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
    ];

    match instruction {
        JupiterSwapInstruction::Route {
            in_token,
            out_token,
            slippage_bps,
        }
        | JupiterSwapInstruction::ExactOutRoute {
            in_token,
            out_token,
            slippage_bps,
        }
        | JupiterSwapInstruction::SharedAccountsRoute {
            in_token,
            out_token,
            slippage_bps,
        } => {
            // Add input token fields
            if let Some(token) = in_token {
                fields.extend([
                    create_text_field("Input Token", &token.symbol)
                        .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
                    create_amount_field("Input Amount", &token.amount.to_string(), &token.symbol)
                        .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
                    create_text_field("Input Token Name", &token.name)
                        .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
                    create_text_field("Input Token Address", &token.address)
                        .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
                ]);
            }

            // Add output token fields
            if let Some(token) = out_token {
                fields.extend([
                    create_text_field("Output Token", &token.symbol)
                        .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
                    create_amount_field(
                        "Quoted Output Amount",
                        &token.amount.to_string(),
                        &token.symbol,
                    )
                    .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
                    create_text_field("Output Token Name", &token.name)
                        .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
                    create_text_field("Output Token Address", &token.address)
                        .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
                ]);
            }

            // Add slippage field
            fields.push(
                create_number_field("Slippage", &slippage_bps.to_string(), "bps")
                    .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
            );
        }
        JupiterSwapInstruction::Unknown => {
            fields.push(
                create_text_field("Status", "Unknown Jupiter instruction type")
                    .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
            );
        }
    }

    // Add raw data field
    fields.push(
        create_raw_data_field(data, Some(hex::encode(data)))
            .map_err(|e| VisualSignError::ConversionError(e.to_string()))?,
    );

    Ok(fields)
}
