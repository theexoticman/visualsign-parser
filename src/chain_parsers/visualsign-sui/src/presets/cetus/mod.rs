use crate::core::{CommandVisualizer, VisualizerContext};
use crate::utils::{Coin, create_address_field, decode_number, get_index};

use sui_json_rpc_types::{SuiArgument, SuiCallArg, SuiCommand};

use visualsign::{
    SignablePayloadField, SignablePayloadFieldCommon, SignablePayloadFieldListLayout,
    field_builders::{create_amount_field, create_text_field},
};

pub const CETUS_AMM_SWAP_B2A_PACKAGE: &str =
    "0xb2db7142fa83210a7d78d9c12ac49c043b3cbbd482224fea6e3da00aa5a5ae2d";
pub const MODULE_POOL_SCRIPT_V2: &str = "pool_script_v2";
pub const FUNC_SWAP_B2A: &str = "swap_b2a";

const ARG_INDEX_INPUT_AMOUNT: usize = 5;
const ARG_INDEX_MIN_OUTPUT: usize = 6;

pub struct CetusVisualizer;

impl CommandVisualizer for CetusVisualizer {
    fn visualize_tx_commands(&self, context: &VisualizerContext) -> Option<SignablePayloadField> {
        let Some(SuiCommand::MoveCall(pwc)) = context.commands().get(context.command_index())
        else {
            return None;
        };

        match pwc.function.as_str() {
            FUNC_SWAP_B2A => {
                let input_coin = get_token_1_coin(&pwc.type_arguments).unwrap_or_default();
                let output_coin = get_token_2_coin(&pwc.type_arguments).unwrap_or_default();

                let input_amount =
                    get_amount_by_index(context.inputs(), &pwc.arguments, ARG_INDEX_INPUT_AMOUNT)
                        .unwrap_or_default();
                let min_output_amount =
                    get_amount_by_index(context.inputs(), &pwc.arguments, ARG_INDEX_MIN_OUTPUT)
                        .unwrap_or_default();

                Some(SignablePayloadField::ListLayout {
                    common: SignablePayloadFieldCommon {
                        fallback_text: "CetusAMM Swap Command".to_string(),
                        label: "CetusAMM Swap Command".to_string(),
                    },
                    list_layout: SignablePayloadFieldListLayout {
                        fields: vec![
                            create_address_field(
                                "From",
                                &context.sender().to_string(),
                                None,
                                None,
                                None,
                                None,
                            ),
                            create_address_field(
                                "To",
                                &context.sender().to_string(),
                                None,
                                None,
                                None,
                                None,
                            ),
                            create_amount_field(
                                "Input Amount",
                                &input_amount.to_string(),
                                input_coin.label(),
                            ),
                            create_text_field("Input Coin", input_coin.label()),
                            create_amount_field(
                                "Min Output Amount",
                                &min_output_amount.to_string(),
                                output_coin.label(),
                            ),
                            create_text_field("Output Coin", output_coin.label()),
                        ],
                    },
                })
            }
            _ => None,
        }
    }

    fn can_handle(&self, context: &VisualizerContext) -> bool {
        if let Some(SuiCommand::MoveCall(pwc)) = context.commands().get(context.command_index()) {
            pwc.package.to_hex_literal() == CETUS_AMM_SWAP_B2A_PACKAGE
                && pwc.module == MODULE_POOL_SCRIPT_V2
                && matches!(pwc.function.as_str(), FUNC_SWAP_B2A)
        } else {
            false
        }
    }
}

fn get_token_1_coin(type_args: &[String]) -> Option<Coin> {
    type_args
        .first()
        .and_then(|coin_type| coin_type.parse().ok())
}

fn get_token_2_coin(type_args: &[String]) -> Option<Coin> {
    type_args
        .get(1)
        .and_then(|coin_type| coin_type.parse().ok())
}

fn get_amount_by_index(
    inputs: &[SuiCallArg],
    args: &[SuiArgument],
    arg_index: usize,
) -> Option<u64> {
    decode_number::<u64>(inputs.get(get_index(args, Some(arg_index))? as usize)?)
}

#[cfg(test)]
mod tests {
    use crate::transaction_string_to_visual_sign;
    use visualsign::vsptrait::VisualSignOptions;

    const CETUS_SWAP_LABEL: &str = "CetusAMM Swap Command";

    #[test]
    fn test_cetus_amm_swap_b2a_commands() {
        // https://suivision.xyz/txblock/7Je4yeXMvvEHFcRSTD4WYv3eSsaDk2zqvdoSxWXdUYGx
        let test_data = "AQAAAAAACQEAEXs/ewhS1RZrUZQ2xQEliCJn40SK4PvEV75r2SGFMXhjUsAjAAAAACBSKqlrLdPXYeuzckz31NAkeSO09qmNPv/pkWggJMTC2QAIuMbAAQAAAAABAdqkYpJjLDxNjzHyPqD5s2oo/zZ36WhJgORDhAOmej2PLgUYAAAAAAAAAQFK94o+ni1sq8pdp5wea/9ImVZqQhMh/DtaYZZkAXpg1nkOqBoAAAAAAQABAQAIuMbAAQAAAAAACI0+GgMAAAAAABCvMxuoMn+7NbHE/v8AAAAAAQEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABgEAAAAAAAAAAAMCAQAAAQEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgRjb2luBHplcm8BB9ujRnLjDLBlsfk+OrVTGHaP1v72bBWULJ98uEbi+QDnBHVzZGMEVVNEQwAAALLbcUL6gyEKfXjZwSrEnAQ7PLvUgiJP6m49oAqlpa4tDnBvb2xfc2NyaXB0X3YyCHN3YXBfYjJhAgfbo0Zy4wywZbH5Pjq1Uxh2j9b+9mwVlCyffLhG4vkA5wR1c2RjBFVTREMAB7eETiiahBDlD7PKSNaeuc8p4n0iPvkDU/4b2OJ/+PP4BGNvaW4EQ09JTgAJAQIAAQMAAgEAAgAAAQQAAQUAAQYAAQcAAQgArltnUkfA5IdctLm9N6YO1bz4kng0TThA3StCbiinZoUBZI8YcdbCiGOtIFCZV/M9U6lZTgf3lg6t7feHRsBBqR1jUsAjAAAAACCmwR6aeqn8D632smpzU9fbDhP3vPOQhgc806IrzekPH65bZ1JHwOSHXLS5vTemDtW8+JJ4NE04QN0rQm4op2aFBQIAAAAAAAC8YDQAAAAAAAABYQAdbFpPHuOPe/TYRMttj4FSzAN1ErZdI75GooTkFmiIVkvCM+lnSS3pR/qQt6j7K3gsrtBExfgOL/dffWapvuMEyeP1ig9kZWEaY4lMw99QxRTo2PcUhKsb1gquOOAGXP8=";

        let payload = transaction_string_to_visual_sign(
            test_data,
            VisualSignOptions {
                decode_transfers: true,
                transaction_name: None,
            },
        )
        .expect("Failed to visualize tx commands");

        payload
            .fields
            .iter()
            .find(|f| f.label() == CETUS_SWAP_LABEL)
            .expect(&format!("Should have a field labeled '{CETUS_SWAP_LABEL}'"));
    }
}
