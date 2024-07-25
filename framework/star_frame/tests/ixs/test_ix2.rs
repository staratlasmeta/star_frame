use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::mutable::Writable;
use star_frame::account_set::signer::Signer;
use star_frame::account_set::AccountSet;
use star_frame::idl::AccountSetToIdl;
use star_frame::instruction::FrameworkInstruction;
use star_frame::sys_calls::SysCallInvoke;
use star_frame::Result;
use star_frame_proc::InstructionToIdl;

#[derive(Copy, Clone, Debug, InstructionToIdl, BorshSerialize, BorshDeserialize)]
pub struct TestInstruction2 {
    pub val: u32,
    pub val2: u64,
    pub val3: Pubkey,
}

impl FrameworkInstruction for TestInstruction2 {
    type SelfData<'a> = Self;
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = TestInstruction2;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = TestInstruction2Accounts<'b, 'info> where 'info: 'b;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        <Self as BorshDeserialize>::deserialize(bytes).map_err(Into::into)
    }

    fn split_to_args<'a>(r: &'a Self::SelfData<'_>) -> SplitToArgsReturn<'a, Self> {
        ((), (), *r, ())
    }

    fn run_instruction<'b, 'info>(
        _run_arg: Self::RunArg<'_>,
        _program_id: &Pubkey,
        _account_set: &mut Self::Accounts<'b, '_, 'info>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
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
