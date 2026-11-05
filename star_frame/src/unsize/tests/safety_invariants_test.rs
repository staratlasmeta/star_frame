use crate::{prelude::*, unsize::test_helpers::TestByteSet, Result};

#[unsized_type(skip_idl)]
struct MemmoveInvariant {
    #[unsized_start]
    before: List<u8>,
    middle: List<u8>,
    after: List<u8>,
}

#[test]
fn memmove_insert_remove_before_sibling_preserves_layout() -> Result<()> {
    let data_set = TestByteSet::<MemmoveInvariant>::new(MemmoveInvariantOwned {
        before: vec![1, 2],
        middle: vec![10, 11],
        after: vec![20, 21, 22],
    })?;
    let mut data = data_set.data_mut()?;

    data.before().insert_all(1, [3, 4, 5])?;
    assert_eq!(&**data.middle, &[10, 11]);
    assert_eq!(&**data.after, &[20, 21, 22]);

    data.before().remove_range(2..4)?;
    assert_eq!(&**data.middle, &[10, 11]);
    assert_eq!(&**data.after, &[20, 21, 22]);

    data.middle().push(12)?;
    data.after().insert(1, 99)?;
    drop(data);

    assert_eq!(
        data_set.owned()?,
        MemmoveInvariantOwned {
            before: vec![1, 3, 2],
            middle: vec![10, 11, 12],
            after: vec![20, 99, 21, 22],
        }
    );

    Ok(())
}

#[unsized_type(skip_idl)]
struct EnumPayload {
    #[unsized_start]
    bytes: List<u8>,
}

#[unsized_type(skip_idl)]
#[repr(u8)]
enum StartPointerEnum {
    #[default_init]
    Payload(EnumPayload),
    Alternate(List<u8>) = 2,
    Empty,
}

#[unsized_type(skip_idl)]
struct EnumStartPointerInvariant {
    #[unsized_start]
    prefix: List<u8>,
    mid: StartPointerEnum,
    suffix: List<u8>,
}

#[test]
fn resize_notification_tracks_enum_start_pointer() -> Result<()> {
    let data_set = TestByteSet::<EnumStartPointerInvariant>::new(EnumStartPointerInvariantOwned {
        prefix: vec![1, 2],
        mid: StartPointerEnumOwned::Payload(EnumPayloadOwned {
            bytes: vec![30, 31],
        }),
        suffix: vec![40],
    })?;
    let mut data = data_set.data_mut()?;

    data.prefix().insert_all(1, [7, 8, 9])?;
    if let StartPointerEnumExclusive::Payload(mut payload) = data.mid().get() {
        payload.bytes().push(32)?;
    } else {
        panic!("Expected payload variant");
    }
    data.suffix().push(41)?;

    data.prefix().remove_range(0..2)?;
    if let StartPointerEnumExclusive::Payload(mut payload) = data.mid().get() {
        payload.bytes().insert(0, 29)?;
    } else {
        panic!("Expected payload variant");
    }
    data.suffix().insert(1, 42)?;
    drop(data);

    assert_eq!(
        data_set.owned()?,
        EnumStartPointerInvariantOwned {
            prefix: vec![8, 9, 2],
            mid: StartPointerEnumOwned::Payload(EnumPayloadOwned {
                bytes: vec![29, 30, 31, 32],
            }),
            suffix: vec![40, 42, 41],
        }
    );

    Ok(())
}
