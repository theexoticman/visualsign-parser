use move_core_types::runtime_value::MoveValue;
use std::fmt::Display;

use sui_json::{MoveTypeLayout, SuiJsonValue};
use sui_json_rpc_types::SuiArgument::Input;
use sui_json_rpc_types::{SuiArgument, SuiProgrammableTransactionBlock};


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

impl Default for CoinObject {
    fn default() -> CoinObject {
        CoinObject::Unknown(String::default())
    }
}

/// Extract amount from SUI arguments in a transaction
pub fn get_amount(
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

/// Get index from SUI arguments array (expects single argument)
pub fn get_index(sui_args: &[SuiArgument]) -> Option<u16> {
    if sui_args.len() != 1 {
        return None;
    }

    parse_numeric_argument(sui_args.first()?)
}

/// Parse numeric argument from SUI argument (Input or Result)
pub fn parse_numeric_argument(arg: &SuiArgument) -> Option<u16> {
    match arg {
        Input(index) => Some(*index),
        SuiArgument::Result(index) => Some(*index),
        _ => None,
    }
}