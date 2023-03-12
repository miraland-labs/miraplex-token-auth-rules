use std::fmt::Display;

/// See state module for description of PDA memory layout.
use crate::error::RuleSetError;
use borsh::BorshSerialize;
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use super::{RuleV2, HEADER_SECTION};

/// Version of the `RuleSetRevisionMapV1` struct.
pub const RULE_SET_REV_MAP_VERSION: u8 = 1;

/// Version of the `RuleSetV2` struct.
pub const RULE_SET_LIB_VERSION: u8 = 2;

pub const NAME_LENGTH: usize = 32;

/// Size of `RuleSetHeader` when Borsh serialized.
pub const RULE_SET_SERIALIZED_HEADER_LEN: usize = 8;

/// The struct containing all Rule Set data, most importantly the map of operations to `Rules`.
///  See top-level module for description of PDA memory layout.
pub struct RuleSetV2<'a> {
    /// Version of the RuleSet.  This is not a user version, but the version
    /// of this lib, to make sure that a `RuleSet` passed into our handlers
    /// is one we are compatible with.
    pub lib_version: &'a u64,
    /// Owner (creator) of the RuleSet.
    pub owner: &'a Pubkey,
    /// Name of the RuleSet, used in PDA derivation.
    pub rule_set_name: &'a Name,
    /// Number of operations on the rule set.
    pub count: &'a u64,
    /// Operations available.
    pub operations: &'a [Name],
    /// Rules for each operation.
    pub rules: Vec<RuleV2<'a>>,
}

impl<'a> RuleSetV2<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        // lib version
        let lib_version = bytemuck::from_bytes::<u64>(&bytes[..std::mem::size_of::<u64>()]);
        let mut cursor = std::mem::size_of::<u64>();

        // owner
        let owner = bytemuck::from_bytes::<Pubkey>(&bytes[cursor..cursor + 32]);
        cursor += 32;

        // name
        let rule_set_name = bytemuck::from_bytes::<Name>(&bytes[cursor..cursor + NAME_LENGTH]);
        cursor += NAME_LENGTH;

        // count
        let count =
            bytemuck::from_bytes::<u64>(&bytes[cursor..cursor + std::mem::size_of::<u64>()]);
        cursor += std::mem::size_of::<u64>();

        // operations
        let slice_end = cursor
            + NAME_LENGTH
                .checked_mul(*count as usize)
                .ok_or(RuleSetError::NumericalOverflow)?;
        let operations = bytemuck::cast_slice(&bytes[cursor..slice_end]);
        cursor = slice_end;

        // rules
        let mut rules = Vec::with_capacity(*count as usize);

        for _ in 0..*count {
            let rule = RuleV2::from_bytes(&bytes[cursor..]).unwrap();
            cursor += HEADER_SECTION + rule.header.length() as usize;
            rules.push(rule);
        }

        Ok(Self {
            lib_version,
            owner,
            rule_set_name,
            count,
            operations,
            rules,
        })
    }

    pub fn serialize(
        owner: Pubkey,
        name: &str,
        operations: &[String],
        rules: &[Vec<u8>],
    ) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();

        // lib version
        let lib_version = RULE_SET_LIB_VERSION as u64;
        BorshSerialize::serialize(&lib_version, &mut data)?;

        // owner
        BorshSerialize::serialize(&owner, &mut data)?;

        // name
        let mut field_bytes = [0u8; NAME_LENGTH];
        field_bytes[..name.len()].copy_from_slice(name.as_bytes());
        BorshSerialize::serialize(&field_bytes, &mut data)?;

        // count
        let count = operations.len() as u64;
        BorshSerialize::serialize(&count, &mut data)?;

        // operations
        operations.iter().for_each(|x| {
            let mut field_bytes = [0u8; NAME_LENGTH];
            field_bytes[..x.len()].copy_from_slice(x.as_bytes());
            BorshSerialize::serialize(&field_bytes, &mut data).unwrap();
        });

        // rules
        rules.iter().for_each(|x| data.extend(x.iter()));

        Ok(data)
    }
}

impl<'a> Display for RuleSetV2<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&format!("RuleSet: {} {{\n", self.rule_set_name))?;
        formatter.write_str("    operations: {\n")?;

        for i in 0..*self.count {
            formatter.write_str(&format!(
                "        \"{}\": {:#}\n",
                self.operations[i as usize], self.rules[i as usize]
            ))?;
        }

        formatter.write_str("}")
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Name {
    pub value: [u8; NAME_LENGTH],
}

impl Display for Name {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = String::from_utf8(self.value.to_vec()).unwrap();
        formatter.write_str(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::bytemuck::{Amount, CompareOp, ProgramOwnedList, RuleSetV2};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_create_amount() {
        // amount rule
        let amount = Amount::serialize(1, CompareOp::Eq, String::from("Destination")).unwrap();

        // program owned rule
        let programs = &[Pubkey::default(), Pubkey::default()];

        let program_owned =
            ProgramOwnedList::serialize(String::from("Destination"), programs).unwrap();

        // rule set

        let serialized = RuleSetV2::serialize(
            Pubkey::default(),
            "Royalties",
            &["deletage_transfer".to_string(), "transfer".to_string()],
            &[amount, program_owned],
        )
        .unwrap();

        // loads a rule set object

        let rule_set = RuleSetV2::from_bytes(&serialized).unwrap();
        println!("{}", rule_set);

        //assert_eq!(rule.header.length(), 32);
    }
}
