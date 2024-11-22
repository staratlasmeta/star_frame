use arrayref::array_refs;
use star_frame::prelude::Pubkey;

pub(crate) fn unpack_option_key(src: &[u8; 36]) -> Option<Pubkey> {
    let (tag, body) = array_refs![src, 4, 32];
    if tag[0] == 1 {
        Some(Pubkey::new_from_array(*body))
    } else {
        None
    }
}

pub(crate) fn unpack_option_u64(src: &[u8; 12]) -> Option<u64> {
    let (tag, body) = array_refs![src, 4, 8];
    if tag[0] == 1 {
        Some(u64::from_le_bytes(*body))
    } else {
        None
    }
}
