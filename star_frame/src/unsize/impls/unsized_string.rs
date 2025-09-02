use crate::prelude::*;
use crate::unsize::FromOwned;
use star_frame::unsize::impls::ListLength;

#[unsized_type(skip_idl, owned_type = String, owned_from_ref = unsized_string_owned_from_ref)]
pub struct UnsizedString<L = u32>
where
    L: ListLength,
{
    #[unsized_start]
    chars: List<u8, L>,
}
#[unsized_impl]
impl<L> UnsizedString<L>
where
    L: ListLength,
{
    pub fn as_str(&self) -> Result<&str> {
        Ok(std::str::from_utf8(self.chars.as_slice())?)
    }

    pub fn as_mut_str(&mut self) -> Result<&mut str> {
        Ok(std::str::from_utf8_mut(self.chars.as_mut_slice())?)
    }

    #[exclusive]
    pub fn set(&mut self, s: impl AsRef<str>) -> Result<()> {
        let mut chars = self.chars();
        chars.clear()?;
        chars.push_all(s.as_ref().as_bytes().iter().copied())?;
        Ok(())
    }
}

fn unsized_string_owned_from_ref<L>(r: &UnsizedStringRef<'_, L>) -> Result<String>
where
    L: ListLength,
{
    r.as_str().map(ToOwned::to_owned)
}

impl<L> FromOwned for UnsizedString<L>
where
    L: ListLength,
{
    fn byte_size(owned: &Self::Owned) -> usize {
        List::<u8, L>::byte_size_from_len(owned.len())
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        List::<u8, L>::from_owned_from_iter(owned.bytes(), bytes)
    }
}
