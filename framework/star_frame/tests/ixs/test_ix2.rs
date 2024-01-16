use bytemuck::{Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::mutable::Writable;
use star_frame::account_set::signer::Signer;
use star_frame::account_set::AccountSet;
use star_frame::idl::AccountSetToIdl;
use star_frame::instruction::FrameworkInstruction;
use star_frame::sys_calls::SysCallInvoke;
use star_frame::Result;
use star_frame_proc::Align1;

#[derive(FrameworkInstruction, Align1, Pod, Zeroable, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct TestInstruction2 {
    pub val: u32,
    pub val2: u64,
    pub val3: Pubkey,
}

#[automatically_derived]
impl<'a> FrameworkInstruction<'a> for &'a TestInstruction2 {
    type DecodeArg = ();
    type ValidateArg = ();
    type RunArg = &'a TestInstruction2;
    type CleanupArg = ();
    type ReturnType = ();
    type Accounts<'b, 'info> = TestInstruction2Accounts<'b, 'info> where 'info: 'b;

    fn split_to_args(
        self,
    ) -> (
        Self::DecodeArg,
        Self::ValidateArg,
        Self::RunArg,
        Self::CleanupArg,
    ) {
        ((), (), self, ())
    }

    fn run_instruction(
        run_arg: Self::RunArg,
        program_id: &Pubkey,
        account_set: &Self::Accounts<'_, '_>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType> {
        todo!()
    }
}

#[derive(AccountSet)]
pub struct TestInstruction2Accounts<'a, 'info>
where
    'info: 'a,
{
    pub signer: Signer<Writable<&'a AccountInfo<'info>>>,
}
