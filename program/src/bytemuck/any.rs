use std::fmt::Display;

use borsh::BorshSerialize;

use crate::{bytemuck::HEADER_SECTION, error::RuleSetError};

use super::{AssertType, Assertable, RuleV2};

pub struct Any<'a> {
    pub size: &'a u64,
    pub rules: Vec<RuleV2<'a>>,
}

impl<'a> Any<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (size, data) = bytes.split_at(std::mem::size_of::<u64>());
        let size = bytemuck::from_bytes::<u64>(size);

        let mut rules = Vec::with_capacity(*size as usize);
        let mut offset = 0;

        for _ in 0..*size {
            let rule = RuleV2::from_bytes(&data[offset..])?;
            offset += HEADER_SECTION + rule.header.length();
            rules.push(rule);
        }

        Ok(Self { size, rules })
    }

    pub fn serialize(rules: &[&[u8]]) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();

        // (Header) rule type
        let rule_type = AssertType::Any as u32;
        BorshSerialize::serialize(&rule_type, &mut data)?;

        // (Header) length
        let length = 8 + rules
            .iter()
            .map(|v| v.len())
            .reduce(|accum, item| accum + item)
            .ok_or(RuleSetError::DataIsEmpty)
            .unwrap() as u32;
        BorshSerialize::serialize(&length, &mut data)?;

        // size
        let size = rules.len() as u64;
        BorshSerialize::serialize(&size, &mut data)?;

        // rules
        rules.iter().for_each(|x| data.extend(x.iter()));

        Ok(data)
    }
}

impl<'a> Assertable<'a> for Any<'a> {
    fn assert_type(&self) -> AssertType {
        AssertType::Any
    }
}

impl<'a> Display for Any<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Any {\n")?;
        formatter.write_str("    rules: [\n")?;

        for p in &self.rules {
            formatter.write_str(&format!("    {}\n", p))?;
        }

        formatter.write_str("    ]\n")?;
        formatter.write_str("}")
    }
}
