use borsh::{BorshDeserialize, BorshSerialize};
use derive_more::{Deref, DerefMut, From, Into};
use std::io::{Read, Write};

/// A helper struct for Borsh that consumes the remaining bytes in a buffer. This is most useful for replicating remaining
/// data in an instruction without the 4 byte length overhead for [`borsh`]'s serialize and deserialize on `Vec`.
#[derive(
    Debug, Clone, PartialEq, Eq, Deref, DerefMut, Default, Hash, Ord, PartialOrd, From, Into,
)]
#[repr(transparent)]
pub struct RemainingData(Vec<u8>);

impl BorshDeserialize for RemainingData {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut data = vec![];
        reader.read_to_end(&mut data)?;
        Ok(Self(data))
    }
}

impl BorshSerialize for RemainingData {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0)
    }
}
