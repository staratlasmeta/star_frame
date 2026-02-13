use crate::prelude::*;

/// An [`InstructionSet`] that errors when called.
#[derive(Align1, Debug, Copy, Clone)]
pub struct UnCallable;

impl InstructionSet for UnCallable {
    type Discriminant = ();

    fn dispatch(_program_id: &Pubkey, _accounts: &[AccountInfo], _ix_bytes: &[u8]) -> Result<()> {
        bail!(
            ProgramError::InvalidInstructionData,
            "Cannot call dispatch on UnCallable"
        )
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::IdlDefinition;

    impl InstructionSetToIdl for UnCallable {
        fn instruction_set_to_idl(_idl_definition: &mut IdlDefinition) -> crate::IdlResult<()> {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static PROGRAM_ID: Pubkey = Pubkey::new_from_array([9; 32]);

    #[test]
    fn un_callable_dispatch_is_non_panicking_and_fails_closed() {
        let dispatch = std::panic::catch_unwind(|| UnCallable::dispatch(&PROGRAM_ID, &[], &[]));
        assert!(dispatch.is_ok());

        let Ok(call_result) = dispatch else {
            unreachable!("UnCallable::dispatch unexpectedly panicked");
        };
        assert!(call_result.is_err());

        let Err(err) = call_result else {
            unreachable!("UnCallable::dispatch unexpectedly returned Ok(())");
        };
        assert_eq!(
            ProgramError::from(err),
            ProgramError::InvalidInstructionData
        );
    }
}
