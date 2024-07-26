use bytemuck::{Pod, Zeroable};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::AccountSet;
use star_frame::align1::Align1;
use star_frame::idl::AccountSetToIdl;
use star_frame::impls::option::Remaining;
use star_frame::instruction::FrameworkInstruction;
use star_frame::instruction::InstructionToIdl;
use star_frame::prelude::SystemAccount;
use star_frame::serialize::FrameworkFromBytes;
use star_frame::syscalls::SysCallInvoke;
use star_frame::Result;

#[derive(Copy, Clone, Zeroable, Align1, Pod, InstructionToIdl)]
#[repr(C, packed)]
pub struct TestInstruction1 {
    /// The first value
    pub val: u32,
    /// The second Value
    pub val2: u64,
    /// The third value
    pub val3: i8,
}

impl FrameworkInstruction for TestInstruction1 {
    type SelfData<'a> = <Self as UnsizedType>::Ref<'a>;
    type DecodeArg<'a> = i8;
    type ValidateArg<'a> = u64;
    type RunArg<'a> = i8;
    type CleanupArg<'a> = (u32, u64);
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = TestInstruction1Accounts<'b, 'info> where 'info: 'b;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        <Self as UnsizedType>::Ref::from_bytes(bytes)
    }

    fn split_to_args<'a>(r: &'a Self::SelfData<'_>) -> SplitToArgsReturn<'a, Self> {
        (r.val3, r.val2, r.val3, (r.val, r.val2))
    }

    fn run_instruction<'b, 'info>(
        _run_arg: Self::RunArg<'_>,
        _program_id: &Pubkey,
        _account_set: &mut Self::Accounts<'b, '_, 'info>,
        _syscalls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
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
    pub account3: SystemAccount<'info>,
}

#[derive(AccountSet)]
pub struct AccountStuff {}
