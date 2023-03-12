use borsh::BorshSerialize;
use std::fmt::Display;

use super::{AssertType, Assertable, CompareOp, FIELD_LENGTH};
use crate::error::RuleSetError;

pub struct Amount<'a> {
    pub amount: &'a u64,
    pub operator: &'a u64,
    pub field: &'a [u8],
}

impl<'a> Amount<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        // amount
        let amount = bytemuck::from_bytes::<u64>(&bytes[..std::mem::size_of::<u64>()]);
        let mut cursor = std::mem::size_of::<u64>();

        // operator
        let operator =
            bytemuck::from_bytes::<u64>(&bytes[cursor..cursor + std::mem::size_of::<u64>()]);
        cursor += std::mem::size_of::<u64>();

        // field
        let field = bytemuck::cast_slice(&bytes[cursor..]);

        Ok(Self {
            amount,
            operator,
            field,
        })
    }

    pub fn serialize(amount: u64, operator: CompareOp, field: String) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();

        // (Header) rule type
        let rule_type = AssertType::Amount as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;

        // (Header) length
        let length = (8 + 8 + FIELD_LENGTH) as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // amount
        BorshSerialize::serialize(&amount, &mut data)?;

        // rules
        let operator = operator as u64;
        BorshSerialize::serialize(&operator, &mut data)?;

        // field
        let mut field_bytes = [0u8; FIELD_LENGTH];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        Ok(data)
    }
}

impl<'a> Assertable<'a> for Amount<'a> {
    fn assert_type(&self) -> AssertType {
        AssertType::Amount
    }
}

impl<'a> Display for Amount<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Amount {\n")?;
        formatter.write_str(&format!("    amount: {},\n", self.amount))?;
        formatter.write_str(&format!("    operator: {}\n", self.operator))?;
        let field = String::from_utf8(self.field.to_vec()).unwrap();
        formatter.write_str(&format!("    field: \"{}\",\n", field))?;
        formatter.write_str("}")
    }
}
