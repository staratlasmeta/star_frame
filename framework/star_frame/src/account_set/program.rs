use crate::prelude::*;
use std::marker::PhantomData;

#[derive(AccountSet, Debug, Clone)]
#[account_set(skip_default_idl)]
#[validate(
    generics = [where T: StarFrameProgram],
    extra_validation = self.check_id(),
)]
pub struct Program<'info, T>(
    #[single_account_set] pub(crate) AccountInfo<'info>,
    pub(crate) PhantomData<T>,
);

impl<T: StarFrameProgram> Program<'_, T> {
    pub fn check_id(&self) -> Result<()> {
        if self.0.key() == &T::PROGRAM_ID {
            Ok(())
        } else {
            Err(ProgramError::IncorrectProgramId.into())
        }
    }
}

// TODO: maybe add some helper methods here? Anchor has a program executable pda find method. Could be helpful to have here too.

#[cfg(feature = "idl")]
mod idl_impl {
    use crate::account_set::Program;
    use crate::idl::AccountSetToIdl;
    use star_frame::prelude::StarFrameProgram;
    use star_frame_idl::account_set::{IdlAccountSetDef, IdlSingleAccountSet};
    use star_frame_idl::IdlDefinition;

    impl<'info, T: StarFrameProgram> AccountSetToIdl<'info, ()> for Program<'info, T> {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            _arg: (),
        ) -> anyhow::Result<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                program_accounts: vec![],
                seeds: None,
                address: Some(T::PROGRAM_ID),
                writable: false,
                signer: false,
                optional: false,
            }))
        }
    }
}
