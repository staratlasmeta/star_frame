use crate::{instruction::IxArgs, prelude::*};

impl InstructionSet for () {
    type Discriminant = ();

    fn dispatch(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _ix_bytes: &[u8],
        _ctx: &mut Context,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

impl InstructionArgs for () {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type CleanupArg<'a> = ();
    type RunArg<'a> = ();

    fn split_to_args(_r: &mut Self) -> IxArgs<'_, Self> {
        IxArgs {
            decode: (),
            validate: (),
            cleanup: (),
            run: (),
        }
    }
}

impl StarFrameInstruction for () {
    type ReturnType = ();
    type Accounts<'decode, 'arg> = ();

    fn process(
        _accounts: &mut Self::Accounts<'_, '_>,
        _run_arg: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        Ok(())
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::idl::InstructionSetToIdl;
    use star_frame_idl::IdlDefinition;

    impl InstructionSetToIdl for () {
        fn instruction_set_to_idl(_idl_definition: &mut IdlDefinition) -> anyhow::Result<()> {
            Ok(())
        }
    }
}
