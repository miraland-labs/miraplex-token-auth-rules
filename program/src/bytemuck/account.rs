//! RuleSetV2 PDA data layout
//! ```text
//! | Header  | RuleSet version | RuleSet Revision 0 | ... | RuleSetRevisionMap |
//! |---------|-----------------|--------------------|-----|--------------------|
//! | 8 bytes | variable bytes  | variable bytes     | ... | variable bytes     |

use bytemuck::{Pod, Zeroable};

use super::SIZE_U64;

/// Account hedaer size in bytes.
pub const ACCOUNT_HEADER_LENGTH: usize = 8;

/// Minimum size of the revisions map in bytes.
pub const MINIMUM_REVISION_MAP_LENGTH: usize = 16;

/// Header used to keep track of where RuleSets are stored in the PDA.  This header is meant
/// to be stored at the beginning of the PDA and never be versioned so that it always
/// has the same serialized size.  See top-level module for description of PDA memory layout.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct AccountHeader {
    /// Stored as an array of u32:
    ///   0. The `Key` for this account which identifies it as a `RuleSet` account.
    ///   1. The location of revision map version stored in the PDA.  This is one byte before the
    ///      revision map itself.
    pub data: [u32; 2],
}

impl AccountHeader {
    pub fn key(&self) -> usize {
        self.data[0] as usize
    }

    pub fn map_location(&self) -> usize {
        self.data[1] as usize
    }

    pub fn set_key(&mut self, key: u32) {
        self.data[0] = key;
    }

    pub fn set_map_location(&mut self, map_location: u32) {
        self.data[1] = map_location;
    }
}

pub struct AccountRevisionMap<'a> {
    pub size: &'a mut u64,
    /// `Vec` used to map a `RuleSet` revision number to its location in the PDA.
    pub revisions: &'a mut [Revision],
}

impl<'a> AccountRevisionMap<'a> {
    pub fn from_bytes_mut(bytes: &'a mut [u8]) -> Self {
        let (size, revisions) = bytes.split_at_mut(SIZE_U64);
        let size = bytemuck::from_bytes_mut::<u64>(size);

        let offset = (*size + 1) as usize * std::mem::size_of::<Revision>();
        let revisions = bytemuck::cast_slice_mut(&mut revisions[..offset]);

        Self { size, revisions }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Revision {
    // Stored as an u32 array:
    ///   0. The `Key` for this account which identifies it as a `RuleSet` account.
    ///   1. The location of revision map version stored in the PDA.  This is one byte before the
    ///      revision map itself.
    pub data: [u32; 2],
}

impl Revision {
    pub fn offset(&self) -> usize {
        self.data[0] as usize
    }

    pub fn legnth(&self) -> usize {
        self.data[1] as usize
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.data[0] = offset;
    }

    pub fn set_length(&mut self, length: u32) {
        self.data[1] = length;
    }
}
