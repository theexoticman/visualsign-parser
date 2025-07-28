use std::collections::HashMap;
use std::fmt::Display;

use move_core_types::runtime_value::MoveValue;

use sui_json::{MoveTypeLayout, SuiJsonValue};

use sui_types::base_types::SuiAddress;

use sui_json_rpc_types::SuiArgument::Input;
use sui_json_rpc_types::SuiCommand::{SplitCoins, TransferObjects};
use sui_json_rpc_types::{
    SuiArgument, SuiProgrammableTransactionBlock, SuiTransactionBlockData,
    SuiTransactionBlockDataAPI, SuiTransactionBlockKind,
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
    tx_data: &SuiTransactionBlockData,
) -> Vec<Result<TransferInfo, String>> {
    let SuiTransactionBlockKind::ProgrammableTransaction(transaction) = &tx_data.transaction()
    else {
        return vec![];
    };

    let results: Vec<Option<SplitCoinResult>> = transaction
        .commands
        .iter()
        .enumerate()
        .map(|(command_index, command)| match command {
            SplitCoins(arg, amounts) => get_amount(transaction, amounts)
                .map(|amount| SplitCoinResult::new(amount, get_token(transaction, arg)))
                .ok_or_else(|| format!("Failed to get amount for command {}", command_index))
                .ok(),
            _ => None,
        })
        .collect();

    let aggregated_transfers: HashMap<(SuiAddress, SuiAddress, CoinObject), u64> = transaction
        .commands
        .iter()
        .filter_map(|command| match command {
            TransferObjects(args, arg) => {
                let result_index = get_index(args)?;
                let result = results.get(result_index as usize)?.as_ref()?;
                let recipient = get_recipient(transaction, arg)?;

                let sender = *tx_data.sender();
                let token = result.token.clone();

                Some((sender, recipient, token, result.amount))
            }
            _ => None,
        })
        .fold(
            HashMap::new(),
            |mut acc, (sender, recipient, token, amount)| {
                acc.entry((sender, recipient, token))
                    .and_modify(|existing_amount| *existing_amount += amount)
                    .or_insert(amount);
                acc
            },
        );

    aggregated_transfers
        .into_iter()
        .map(|((sender, recipient, token), amount)| {
            Ok(TransferInfo {
                sender,
                recipient,
                amount,
                coin_object: token,
            })
        })
        .collect()
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

    use move_bytecode_utils::module_cache::SyncModuleCache;
    use std::str::FromStr;
    use visualsign::encodings::SupportedEncodings;

    use crate::module_resolver::SuiModuleResolver;

    #[test]
    fn test_detect_native_transfer() -> anyhow::Result<()> {
        let test_data = "AQAAAAAAAgAI6AMAAAAAAAAAIKHjrlUcKr48a86iLT8ZNWpkcIbWvVasDQnk7u0GKQt2AgIAAQEAAAEBAgAAAQEA1ukuAC4mw6+yCIABwbWCC2TyvDUb/aWiNCrL+fXBysIBy0he+AoLr5B5piHELIsMtlzpmG4cgf0W7ogDjwBKWu3zD9AUAAAAACB0zCGEALsfD5u98y58qbKGIiXkCtDxxN2Pu+r/HyOy1tbpLgAuJsOvsgiAAcG1ggtk8rw1G/2lojQqy/n1wcrC6AMAAAAAAABAS0wAAAAAAAABYQBMegviWYFsLskcYMnTIhZRxiZkET3j2RqtgG1g7f1/EuPjfCHfTvgDqVys+AA6jLWojR35eW4HoOh8qURdshkADNDs6YjOg+HDmdMLe0zMuMDJKqzwIYg08CT6mXiLc2Y=";

        let result = decode_transaction(test_data, SupportedEncodings::Base64).unwrap();
        let block_data = SuiTransactionBlockData::try_from_with_module_cache(
            result.clone(),
            &SyncModuleCache::new(SuiModuleResolver),
        )?;
        let result = detect_transfer_from_transaction(&block_data);

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
        let block_data = SuiTransactionBlockData::try_from_with_module_cache(
            result.clone(),
            &SyncModuleCache::new(SuiModuleResolver),
        )?;
        let result = detect_transfer_from_transaction(&block_data);

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
