use std::fmt::Display;

use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;

use crate::error::RuleSetError;

use super::{AssertType, Assertable, FIELD_LENGTH};

pub struct ProgramOwnedList<'a> {
    pub field: &'a [u8],
    pub programs: &'a [Pubkey],
}

impl<'a> ProgramOwnedList<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (field, programs) = bytes.split_at(FIELD_LENGTH);
        let field = bytemuck::cast_slice(field);
        let programs = bytemuck::cast_slice(programs);

        Ok(Self { field, programs })
    }

    pub fn serialize(field: String, programs: &[Pubkey]) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();

        // (Header) rule type
        let rule_type = AssertType::ProgramOwnedList as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;

        // (Header) length
        let length = FIELD_LENGTH as u32 + (programs.len() * 32) as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // field
        let mut field_bytes = [0u8; FIELD_LENGTH];
        field_bytes[..field.len()].copy_from_slice(field.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        // programs
        programs.iter().for_each(|x| {
            BorshSerialize::serialize(x, &mut data).unwrap();
        });

        Ok(data)
    }
}

impl<'a> Assertable<'a> for ProgramOwnedList<'a> {
    fn assert_type(&self) -> AssertType {
        AssertType::ProgramOwnedList
    }
}

impl<'a> Display for ProgramOwnedList<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ProgramOwnedList {\n")?;
        formatter.write_str("    programs: [\n")?;

        for p in self.programs {
            formatter.write_fmt(format_args!("        {},\n", p))?;
        }

        formatter.write_str("    ],\n")?;
        let field = String::from_utf8(self.field.to_vec()).unwrap();
        formatter.write_str(&format!("    field: \"{}\",\n", field))?;

        formatter.write_str("}")
    }
}
