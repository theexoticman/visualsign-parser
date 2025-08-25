mod config;
pub mod swap;
use crate::core::{
    InstructionVisualizer, SolanaIntegrationConfig, VisualizerContext, VisualizerKind,
};
use config::JupiterConfig;
use visualsign::errors::VisualSignError;
use visualsign::{AnnotatedPayloadField, SignablePayloadField, SignablePayloadFieldCommon};

// Create a static instance that we can reference
static JUPITER_CONFIG: JupiterConfig = JupiterConfig;

pub struct JupiterVisualizer;

impl InstructionVisualizer for JupiterVisualizer {
    fn visualize_tx_commands(
        &self,
        context: &VisualizerContext,
    ) -> Result<AnnotatedPayloadField, VisualSignError> {
        let instruction = context
            .current_instruction()
            .ok_or_else(|| VisualSignError::MissingData("No instruction found".into()))?;

        // Basic Jupiter swap visualization
        let instruction_text = if instruction.data.len() >= 8 {
            let discriminator = &instruction.data[0..8];
            match discriminator {
                [0xc1, 0x20, 0x9b, 0x33, 0x41, 0xd6, 0x9c, 0x81] => "Jupiter Swap".to_string(),
                [0x2a, 0xad, 0xe3, 0x7a, 0x97, 0xcb, 0x17, 0xe5] => "Jupiter Route".to_string(),
                [0x2a, 0xb6, 0xd0, 0x0c, 0xa8, 0xdf, 0xd7, 0x4b] => {
                    "Jupiter Exact Out Route".to_string()
                }
                [0x2a, 0xd4, 0xb6, 0x2f, 0xae, 0xaa, 0xf2, 0x3a] => {
                    "Jupiter Shared Accounts Route".to_string()
                }
                _ => "Jupiter Unknown Instruction".to_string(),
            }
        } else {
            "Jupiter Instruction".to_string()
        };

        Ok(AnnotatedPayloadField {
            static_annotation: None,
            dynamic_annotation: None,
            signable_payload_field: SignablePayloadField::TextV2 {
                common: SignablePayloadFieldCommon {
                    label: "Jupiter".to_string(),
                    fallback_text: instruction_text.clone(),
                },
                text_v2: visualsign::SignablePayloadFieldTextV2 {
                    text: instruction_text,
                },
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
