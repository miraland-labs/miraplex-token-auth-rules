use std::fmt::Display;

use bytemuck::{Pod, Zeroable};

use super::{All, Amount, Any, AssertType, Assertable, ProgramOwnedList};
use crate::error::RuleSetError;

// Size of the header section.
pub const HEADER_SECTION: usize = 8;

pub struct RuleV2<'a> {
    pub header: &'a Header,
    pub data: Box<dyn Assertable<'a> + 'a>,
}

impl<'a> RuleV2<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RuleSetError> {
        let (header, data) = bytes.split_at(HEADER_SECTION);
        let header = bytemuck::from_bytes::<Header>(header);

        let rule_type = header.assert_type();
        let length = header.length();

        let data = match rule_type {
            AssertType::Amount => {
                Box::new(Amount::from_bytes(&data[..length])?) as Box<dyn Assertable>
            }
            AssertType::Any => Box::new(Any::from_bytes(&data[..length])?) as Box<dyn Assertable>,
            AssertType::All => Box::new(All::from_bytes(&data[..length])?) as Box<dyn Assertable>,
            AssertType::ProgramOwnedList => {
                Box::new(ProgramOwnedList::from_bytes(&data[..length])?) as Box<dyn Assertable>
            }
        };

        Ok(Self { header, data })
    }
}

impl<'a> Assertable<'_> for RuleV2<'a> {
    fn assert_type(&self) -> AssertType {
        self.header.assert_type()
    }
}

impl<'a> Display for RuleV2<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("{}", self.data))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Header {
    pub data: [u32; 2],
}

impl Header {
    pub fn assert_type(&self) -> AssertType {
        AssertType::try_from(self.data[0]).unwrap()
    }

    pub fn length(&self) -> usize {
        self.data[1] as usize
    }
}

#[cfg(test)]
mod tests {
    use super::RuleV2;
    use crate::bytemuck::{Amount, Any, CompareOp, ProgramOwnedList, FIELD_LENGTH};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_create_amount() {
        let amount = Amount::serialize(1, CompareOp::Eq, String::from("Destination")).unwrap();

        // loads the data using bytemuck

        let rule = RuleV2::from_bytes(&amount).unwrap();

        assert_eq!(rule.header.length(), 48);
    }

    #[test]
    fn test_create_program_owned_list() {
        let programs = &[Pubkey::default(), Pubkey::default()];

        let program_owned =
            ProgramOwnedList::serialize(String::from("Destination"), programs).unwrap();

        // loads the data using bytemuck

        let rule = RuleV2::from_bytes(&program_owned).unwrap();
        println!("{}", rule);

        assert_eq!(rule.header.length(), 96);
    }

    #[test]
    fn test_create_large_program_owned_list() {
        const SIZE: usize = 1000;

        let mut programs = Vec::new();

        for _ in 0..SIZE {
            programs.push(Pubkey::default());
        }

        let program_owned =
            ProgramOwnedList::serialize(String::from("Destination"), programs.as_mut_slice())
                .unwrap();

        // loads the data using bytemuck

        let rule = RuleV2::from_bytes(&program_owned).unwrap();

        assert_eq!(rule.header.length(), FIELD_LENGTH + (SIZE * 32));
    }

    #[test]
    fn test_create_any() {
        let programs_list1 = &[Pubkey::default()];
        let program_owned1 =
            ProgramOwnedList::serialize(String::from("Destination"), programs_list1).unwrap();

        let programs_list2 = &[Pubkey::default(), Pubkey::default(), Pubkey::default()];
        let program_owned2 =
            ProgramOwnedList::serialize(String::from("Destination"), programs_list2).unwrap();

        let any = Any::serialize(&[&program_owned1, &program_owned2]).unwrap();

        // loads the data using bytemuck
        let rule = RuleV2::from_bytes(&any).unwrap();

        println!("{}", rule);

        assert_eq!(
            rule.header.length(),
            8 + program_owned1.len() + program_owned2.len()
        );
    }
}
