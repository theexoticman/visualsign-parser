use std::convert::TryInto;

use sui_json_rpc_types::SuiCallArg;

use move_core_types::annotated_value::MoveTypeLayout;

#[allow(dead_code)]
pub fn decode_bool(call_arg: &SuiCallArg) -> Option<bool> {
    match call_arg.pure()?.to_bcs_bytes(&MoveTypeLayout::Bool).ok()?[0] {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

// TODO: think about u256
pub fn decode_number<T>(call_arg: &SuiCallArg) -> Option<T>
where
    T: FromLeBytes,
{
    let pure_value = call_arg.pure()?;

    match align_of::<T>() {
        1 => T::from_le_bytes(&pure_value.to_bcs_bytes(&MoveTypeLayout::U8).ok()?),
        2 => T::from_le_bytes(&pure_value.to_bcs_bytes(&MoveTypeLayout::U16).ok()?),
        4 => T::from_le_bytes(&pure_value.to_bcs_bytes(&MoveTypeLayout::U32).ok()?),
        8 => T::from_le_bytes(&pure_value.to_bcs_bytes(&MoveTypeLayout::U64).ok()?),
        16 => T::from_le_bytes(&pure_value.to_bcs_bytes(&MoveTypeLayout::U128).ok()?),
        _ => None,
    }
}

pub trait FromLeBytes: Sized {
    fn from_le_bytes(bytes: &[u8]) -> Option<Self>;
}

impl FromLeBytes for u8 {
    fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
        Some(bytes[0])
    }
}

impl FromLeBytes for u16 {
    fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
        Some(u16::from_le_bytes(bytes.try_into().ok()?))
    }
}

impl FromLeBytes for u32 {
    fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
        Some(u32::from_le_bytes(bytes.try_into().ok()?))
    }
}

impl FromLeBytes for u64 {
    fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
        Some(u64::from_le_bytes(bytes.try_into().ok()?))
    }
}

impl FromLeBytes for u128 {
    fn from_le_bytes(bytes: &[u8]) -> Option<Self> {
        Some(u128::from_le_bytes(bytes.try_into().ok()?))
    }
}
