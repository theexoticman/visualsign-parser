use move_core_types::runtime_value::MoveValue;
use std::fmt::Display;
use sui_json::{MoveTypeLayout, SuiJsonValue};

use sui_json_rpc_types::SuiArgument::{Input, Result};
use sui_json_rpc_types::SuiCommand::{SplitCoins, TransferObjects};
use sui_json_rpc_types::{
    SuiArgument, SuiProgrammableTransactionBlock, SuiTransactionBlock, SuiTransactionBlockDataAPI,
    SuiTransactionBlockKind,
};
use sui_types::base_types::SuiAddress;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Sui,
    USDCTestnet,
    Unknown(String),
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Sui => write!(f, "Sui"),
            TokenType::USDCTestnet => write!(f, "USDCTestnet"),
            TokenType::Unknown(s) => write!(f, "Unknown token: {}", s),
        }
    }
}

impl TokenType {
    pub fn get_label(&self) -> String {
        match self {
            TokenType::Sui => "Sui".to_string(),
            TokenType::USDCTestnet => "USDC Testnet".to_string(),
            TokenType::Unknown(_) => "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub recipient: Option<SuiAddress>,
    pub amount: Option<u64>,
    pub token: TokenType,
}

impl Default for TokenType {
    fn default() -> TokenType {
        TokenType::Unknown(String::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct SplitCoinResult {
    amount: Option<u64>,
    token: TokenType,
}

impl SplitCoinResult {
    fn new(amount: Option<u64>, token: TokenType) -> Self {
        Self { amount, token }
    }
}

pub fn detect_transfer_from_transaction(tx_block: &SuiTransactionBlock) -> Vec<TransferInfo> {
    let tx_data = &tx_block.data;

    let SuiTransactionBlockKind::ProgrammableTransaction(transaction) = tx_data.transaction()
    else {
        return vec![];
    };

    let mut transfers: Vec<TransferInfo> = vec![];
    let mut results = vec![None::<SplitCoinResult>; transaction.commands.len()];

    for (command_index, command) in transaction.commands.iter().enumerate() {
        match command {
            SplitCoins(arg, amounts) => {
                results[command_index] = Some(SplitCoinResult::new(
                    get_amount(transaction, amounts),
                    get_token(transaction, arg),
                ));
            }
            TransferObjects(args, arg) => {
                let Some(result_index) = get_index(args) else {
                    continue;
                };

                let Some(result) = results[result_index as usize].as_ref() else {
                    continue;
                };

                transfers.push(TransferInfo {
                    recipient: get_recipient(transaction, arg),
                    amount: result.amount,
                    token: result.token.clone(),
                });
            }
            _ => {}
        }
    }

    transfers
}

fn get_token(transaction: &SuiProgrammableTransactionBlock, arg: &SuiArgument) -> TokenType {
    match arg {
        SuiArgument::GasCoin => TokenType::Sui,
        Input(index) => {
            let Some(sui_value) = transaction.inputs.get(*index as usize) else {
                return TokenType::Unknown(String::default());
            };

            let Some(coin_object) = sui_value.object() else {
                return TokenType::Unknown("".to_string());
            };

            object_id_to_token_type(&coin_object.to_hex())
        }
        _ => TokenType::Unknown(String::default()),
    }
}

fn object_id_to_token_type(object_id: &str) -> TokenType {
    match object_id {
        "c200eb0447c199d8d54ab891286627d33349fd3d722768e2145b89bf50c4fafa" => {
            TokenType::USDCTestnet
        }
        _ => TokenType::Unknown(format!("Unknown token. Object ID: {}", object_id)),
    }
}

fn get_amount(
    transaction: &SuiProgrammableTransactionBlock,
    sui_args: &[SuiArgument],
) -> Option<u64> {
    let sui_value = transaction.inputs.get(get_index(sui_args)? as usize)?;

    let Ok(MoveValue::U64(decoded_value)) =
        SuiJsonValue::to_move_value(&sui_value.pure()?.to_json_value(), &MoveTypeLayout::U64)
    else {
        return None;
    };

    Some(decoded_value)
}

fn get_recipient(
    transaction: &SuiProgrammableTransactionBlock,
    arg: &SuiArgument,
) -> Option<SuiAddress> {
    let sui_value = transaction
        .inputs
        .get(parse_numeric_argument(arg)? as usize)?;
    sui_value.pure()?.to_sui_address().ok()
}

fn get_index(sui_args: &[SuiArgument]) -> Option<u16> {
    if sui_args.len() != 1 {
        return None;
    }

    parse_numeric_argument(sui_args.first()?)
}

fn parse_numeric_argument(arg: &SuiArgument) -> Option<u16> {
    match arg {
        Input(index) => Some(*index),
        Result(index) => Some(*index),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TransactionEncoding, parse_sui_transaction};
    use std::str::FromStr;

    #[test]
    fn test_detect_native_transfer() -> anyhow::Result<()> {
        let test_data = "AQAAAAAAAgAI6AMAAAAAAAAAIKHjrlUcKr48a86iLT8ZNWpkcIbWvVasDQnk7u0GKQt2AgIAAQEAAAEBAgAAAQEA1ukuAC4mw6+yCIABwbWCC2TyvDUb/aWiNCrL+fXBysIBy0he+AoLr5B5piHELIsMtlzpmG4cgf0W7ogDjwBKWu3zD9AUAAAAACB0zCGEALsfD5u98y58qbKGIiXkCtDxxN2Pu+r/HyOy1tbpLgAuJsOvsgiAAcG1ggtk8rw1G/2lojQqy/n1wcrC6AMAAAAAAABAS0wAAAAAAAABYQBMegviWYFsLskcYMnTIhZRxiZkET3j2RqtgG1g7f1/EuPjfCHfTvgDqVys+AA6jLWojR35eW4HoOh8qURdshkADNDs6YjOg+HDmdMLe0zMuMDJKqzwIYg08CT6mXiLc2Y=";

        let result = parse_sui_transaction(test_data.to_string(), TransactionEncoding::Base64)?;
        let result = detect_transfer_from_transaction(&result);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].token, TokenType::Sui);
        assert_eq!(result[0].amount, Some(1000));
        assert_eq!(
            result[0].recipient,
            Some(SuiAddress::from_str(
                "0xa1e3ae551c2abe3c6bcea22d3f19356a647086d6bd56ac0d09e4eeed06290b76"
            )?)
        );

        Ok(())
    }

    #[test]
    fn test_detect_usdc_transfer() -> anyhow::Result<()> {
        let test_data = "AQAAAAAABAAgoeOuVRwqvjxrzqItPxk1amRwhta9VqwNCeTu7QYpC3YBAMIA6wRHwZnY1Uq4kShmJ9MzSf09cido4hRbib9QxPr6GprUFwAAAAAgaLIB/QqiGeVY7g/t0gmAgBUq5KN1vBtUCNfQl+OWI4QACBAnAAAAAAAAAAggTgAAAAAAAAQCAQEAAQEDAAEBAgAAAQAAAgEBAAEBAgABAQICAAEAANbpLgAuJsOvsgiAAcG1ggtk8rw1G/2lojQqy/n1wcrCActIXvgKC6+QeaYhxCyLDLZc6ZhuHIH9Fu6IA48ASlrtGprUFwAAAAAgruP9lGIbTNb4l4WPdDGN2qrKMg4H7WiVr4iK3KnMEI/W6S4ALibDr7IIgAHBtYILZPK8NRv9paI0Ksv59cHKwugDAAAAAAAAQEtMAAAAAAAAAWEAmEURyDG9UG5JOixWeOweSlyhULQ2oNgiAUrKrio+mjI8yelPjyw5AFA8WOgv9T/RytUNWfnqKsStA67qnisQAwzQ7OmIzoPhw5nTC3tMzLjAySqs8CGINPAk+pl4i3Nm";

        let result = parse_sui_transaction(test_data.to_string(), TransactionEncoding::Base64)?;
        let result = detect_transfer_from_transaction(&result);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].token, TokenType::USDCTestnet);
        assert_eq!(result[0].amount, Some(20000));
        assert_eq!(
            result[0].recipient,
            Some(SuiAddress::from_str(
                "0xa1e3ae551c2abe3c6bcea22d3f19356a647086d6bd56ac0d09e4eeed06290b76"
            )?)
        );

        assert_eq!(result[1].token, TokenType::USDCTestnet);
        assert_eq!(result[1].amount, Some(10000));
        assert_eq!(
            result[1].recipient,
            Some(SuiAddress::from_str(
                "0xa1e3ae551c2abe3c6bcea22d3f19356a647086d6bd56ac0d09e4eeed06290b76"
            )?)
        );

        Ok(())
    }
}
