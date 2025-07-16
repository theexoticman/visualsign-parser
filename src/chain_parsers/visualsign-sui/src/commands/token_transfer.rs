use std::collections::HashMap;
use std::fmt::Display;

use move_core_types::runtime_value::MoveValue;

use sui_json::{MoveTypeLayout, SuiJsonValue};

use sui_types::base_types::SuiAddress;

use sui_json_rpc_types::SuiArgument::Input;
use sui_json_rpc_types::SuiCommand::{SplitCoins, TransferObjects};
use sui_json_rpc_types::{
    SuiArgument, SuiProgrammableTransactionBlock, SuiTransactionBlock, SuiTransactionBlockDataAPI,
    SuiTransactionBlockKind,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoinObject {
    Sui,
    Unknown(String),
}

impl Display for CoinObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoinObject::Sui => write!(f, "Sui"),
            CoinObject::Unknown(s) => write!(f, "Object ID: {}", s),
        }
    }
}

impl CoinObject {
    pub fn get_label(&self) -> String {
        match self {
            CoinObject::Sui => "Sui".to_string(),
            CoinObject::Unknown(_) => "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub sender: SuiAddress,
    pub recipient: SuiAddress,
    pub amount: u64,
    pub coin_object: CoinObject,
}

impl Default for CoinObject {
    fn default() -> CoinObject {
        CoinObject::Unknown(String::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct SplitCoinResult {
    amount: u64,
    token: CoinObject,
}

impl SplitCoinResult {
    fn new(amount: u64, token: CoinObject) -> Self {
        Self { amount, token }
    }
}

pub fn detect_transfer_from_transaction(
    tx_block: &SuiTransactionBlock,
) -> Vec<Result<TransferInfo, String>> {
    let tx_data = &tx_block.data;

    let SuiTransactionBlockKind::ProgrammableTransaction(transaction) = &tx_data.transaction()
    else {
        return vec![];
    };

    let mut transfers: Vec<Result<TransferInfo, String>> = vec![];

    let mut aggregated_transfers: HashMap<(SuiAddress, SuiAddress, CoinObject), u64> =
        HashMap::new();
    let mut results = vec![None::<SplitCoinResult>; transaction.commands.len()];

    for (command_index, command) in transaction.commands.iter().enumerate() {
        match command {
            SplitCoins(arg, amounts) => {
                let Some(amount) = get_amount(transaction, amounts) else {
                    transfers.push(Err(format!(
                        "Failed to get amount for command {}",
                        command_index
                    )));

                    continue;
                };

                results[command_index] =
                    Some(SplitCoinResult::new(amount, get_token(transaction, arg)));
            }
            TransferObjects(args, arg) => {
                let Some(result_index) = get_index(args) else {
                    transfers.push(Err(format!(
                        "Failed to get index for command {}",
                        command_index
                    )));
                    continue;
                };

                let Some(result) = results[result_index as usize].as_ref() else {
                    transfers.push(Err(format!(
                        "Failed to get result for command {}",
                        command_index
                    )));
                    continue;
                };

                let Some(recipient) = get_recipient(transaction, arg) else {
                    transfers.push(Err(format!(
                        "Failed to get recipient for command {}",
                        command_index
                    )));
                    continue;
                };

                let sender = *tx_data.sender();
                let token = result.token.clone();

                if let Some(existing_amount) =
                    aggregated_transfers.get_mut(&(sender, recipient, token.clone()))
                {
                    *existing_amount += result.amount;
                    continue;
                }

                aggregated_transfers.insert((sender, recipient, token.clone()), result.amount);
            }
            _ => {}
        }
    }

    for ((sender, recipient, token), amount) in aggregated_transfers {
        transfers.push(Ok(TransferInfo {
            sender,
            recipient,
            amount,
            coin_object: token.clone(),
        }));
    }

    transfers
}

fn get_token(transaction: &SuiProgrammableTransactionBlock, arg: &SuiArgument) -> CoinObject {
    match arg {
        SuiArgument::GasCoin => CoinObject::Sui,
        Input(index) => {
            let Some(sui_value) = transaction.inputs.get(*index as usize) else {
                return CoinObject::Unknown(String::default());
            };

            let Some(object_id) = sui_value.object() else {
                return CoinObject::Unknown("".to_string());
            };

            CoinObject::Unknown(object_id.to_hex())
        }
        _ => CoinObject::Unknown(String::default()),
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
        SuiArgument::Result(index) => Some(*index),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualsign::decode_transaction;

    use std::str::FromStr;
    use visualsign::encodings::SupportedEncodings;

    #[test]
    fn test_detect_native_transfer() -> anyhow::Result<()> {
        let test_data = "AQAAAAAAAgAI6AMAAAAAAAAAIKHjrlUcKr48a86iLT8ZNWpkcIbWvVasDQnk7u0GKQt2AgIAAQEAAAEBAgAAAQEA1ukuAC4mw6+yCIABwbWCC2TyvDUb/aWiNCrL+fXBysIBy0he+AoLr5B5piHELIsMtlzpmG4cgf0W7ogDjwBKWu3zD9AUAAAAACB0zCGEALsfD5u98y58qbKGIiXkCtDxxN2Pu+r/HyOy1tbpLgAuJsOvsgiAAcG1ggtk8rw1G/2lojQqy/n1wcrC6AMAAAAAAABAS0wAAAAAAAABYQBMegviWYFsLskcYMnTIhZRxiZkET3j2RqtgG1g7f1/EuPjfCHfTvgDqVys+AA6jLWojR35eW4HoOh8qURdshkADNDs6YjOg+HDmdMLe0zMuMDJKqzwIYg08CT6mXiLc2Y=";

        let result = decode_transaction(test_data, SupportedEncodings::Base64).unwrap();
        let result = detect_transfer_from_transaction(&result);

        assert_eq!(result.len(), 1);

        let transfer = result.first().unwrap().as_ref().unwrap();
        assert_eq!(transfer.coin_object, CoinObject::Sui);
        assert_eq!(transfer.amount, 1000);
        assert_eq!(
            transfer.recipient,
            SuiAddress::from_str(
                "0xa1e3ae551c2abe3c6bcea22d3f19356a647086d6bd56ac0d09e4eeed06290b76"
            )?
        );

        Ok(())
    }

    #[test]
    fn test_detect_usdc_transfer() -> anyhow::Result<()> {
        let test_data = "AQAAAAAABAAgoeOuVRwqvjxrzqItPxk1amRwhta9VqwNCeTu7QYpC3YBAMIA6wRHwZnY1Uq4kShmJ9MzSf09cido4hRbib9QxPr6GprUFwAAAAAgaLIB/QqiGeVY7g/t0gmAgBUq5KN1vBtUCNfQl+OWI4QACBAnAAAAAAAAAAggTgAAAAAAAAQCAQEAAQEDAAEBAgAAAQAAAgEBAAEBAgABAQICAAEAANbpLgAuJsOvsgiAAcG1ggtk8rw1G/2lojQqy/n1wcrCActIXvgKC6+QeaYhxCyLDLZc6ZhuHIH9Fu6IA48ASlrtGprUFwAAAAAgruP9lGIbTNb4l4WPdDGN2qrKMg4H7WiVr4iK3KnMEI/W6S4ALibDr7IIgAHBtYILZPK8NRv9paI0Ksv59cHKwugDAAAAAAAAQEtMAAAAAAAAAWEAmEURyDG9UG5JOixWeOweSlyhULQ2oNgiAUrKrio+mjI8yelPjyw5AFA8WOgv9T/RytUNWfnqKsStA67qnisQAwzQ7OmIzoPhw5nTC3tMzLjAySqs8CGINPAk+pl4i3Nm";

        let result = decode_transaction(test_data, SupportedEncodings::Base64).unwrap();
        let result = detect_transfer_from_transaction(&result);

        assert_eq!(result.len(), 1);

        let transfer = result.first().unwrap().as_ref().unwrap();

        assert_eq!(
            transfer.coin_object,
            CoinObject::Unknown(
                "c200eb0447c199d8d54ab891286627d33349fd3d722768e2145b89bf50c4fafa".to_string()
            )
        );
        assert_eq!(transfer.amount, 30000);
        assert_eq!(
            transfer.recipient,
            SuiAddress::from_str(
                "0xa1e3ae551c2abe3c6bcea22d3f19356a647086d6bd56ac0d09e4eeed06290b76"
            )?
        );

        Ok(())
    }
}
