use bytemuck::{Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::AccountSet;
use star_frame::idl::AccountSetToIdl;
use star_frame::impls::option::Remaining;
use star_frame::instruction::FrameworkInstruction;
use star_frame::sys_calls::SysCallInvoke;
use star_frame::Result;
use star_frame_proc::Align1;

#[derive(Copy, Clone, Zeroable, Align1, Pod, FrameworkInstruction)]
#[repr(C, packed)]
pub struct TestInstruction1 {
    /// The first value
    pub val: u32,
    /// The second Value
    pub val2: u64,
    /// The third value
    pub val3: i8,
}

impl<'a> FrameworkInstruction<'a> for &'a TestInstruction1 {
    type DecodeArg = i8;
    type ValidateArg = u64;
    type RunArg = i8;
    type CleanupArg = (u32, u64);
    type ReturnType = ();
    type Accounts<'b, 'info> = TestInstruction1Accounts<'b, 'info> where 'info: 'b;

    fn split_to_args(
        self,
    ) -> (
        Self::DecodeArg,
        Self::ValidateArg,
        Self::RunArg,
        Self::CleanupArg,
    ) {
        (self.val3, self.val2, self.val3, (self.val, self.val2))
    }

    fn run_instruction(
        _run_arg: Self::RunArg,
        _program_id: &Pubkey,
        _account_set: &Self::Accounts<'_, '_>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType> {
        todo!()
    }
}

/// Hello
/// Cioi
#[derive(AccountSet)]
#[decode(arg = i8)]
#[validate(arg = u64)]
#[cleanup(arg = (u32, u64))]
pub struct TestInstruction1Accounts<'a, 'info>
where
    'info: 'a,
{
    pub account1: &'a AccountInfo<'info>,
    #[decode(arg = Remaining(()))]
    pub account2: Option<&'a AccountInfo<'info>>,
}

#[derive(AccountSet)]
pub struct AccountStuff {}
