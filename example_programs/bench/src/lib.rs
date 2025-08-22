use star_frame::{empty_star_frame_instruction, prelude::*};
use star_frame_spl::token::state::{MintAccount, TokenAccount};

#[derive(StarFrameProgram)]
#[program(
    instruction_set = BenchInstructionSet,
    id = "Bench11111111111111111111111111111111111111",
)]
pub struct Bench;

#[derive(InstructionSet)]
pub enum BenchInstructionSet {
    AccountInfo1(AccountInfo1),
    AccountInfo2(AccountInfo2),
    AccountInfo4(AccountInfo4),
    AccountInfo8(AccountInfo8),
    AccountEmptyInit1(AccountEmptyInit1),
    AccountEmptyInit2(AccountEmptyInit2),
    AccountEmptyInit4(AccountEmptyInit4),
    AccountEmptyInit8(AccountEmptyInit8),
    AccountEmpty1(AccountEmpty1),
    AccountEmpty2(AccountEmpty2),
    AccountEmpty4(AccountEmpty4),
    AccountEmpty8(AccountEmpty8),
    AccountSizedInit1(AccountSizedInit1),
    AccountSizedInit2(AccountSizedInit2),
    AccountSizedInit4(AccountSizedInit4),
    AccountSizedInit8(AccountSizedInit8),
    // The following are temporarily disabled during migration to Star Frame pattern
    AccountSized1(AccountSized1),
    AccountSized2(AccountSized2),
    AccountSized4(AccountSized4),
    AccountSized8(AccountSized8),
    AccountUnsizedInit1(AccountUnsizedInit1),
    AccountUnsizedInit2(AccountUnsizedInit2),
    AccountUnsizedInit4(AccountUnsizedInit4),
    AccountUnsizedInit8(AccountUnsizedInit8),
    AccountUnsized1(AccountUnsized1),
    AccountUnsized2(AccountUnsized2),
    AccountUnsized4(AccountUnsized4),
    AccountUnsized8(AccountUnsized8),
    // Boxed groups temporarily disabled until converted
    BoxedAccountEmptyInit1(BoxedAccountEmptyInit1),
    BoxedAccountEmptyInit2(BoxedAccountEmptyInit2),
    BoxedAccountEmptyInit4(BoxedAccountEmptyInit4),
    BoxedAccountEmptyInit8(BoxedAccountEmptyInit8),
    BoxedAccountEmpty1(BoxedAccountEmpty1),
    BoxedAccountEmpty2(BoxedAccountEmpty2),
    BoxedAccountEmpty4(BoxedAccountEmpty4),
    BoxedAccountEmpty8(BoxedAccountEmpty8),
    BoxedAccountSizedInit1(BoxedAccountSizedInit1),
    BoxedAccountSizedInit2(BoxedAccountSizedInit2),
    BoxedAccountSizedInit4(BoxedAccountSizedInit4),
    BoxedAccountSizedInit8(BoxedAccountSizedInit8),
    BoxedAccountSized1(BoxedAccountSized1),
    BoxedAccountSized2(BoxedAccountSized2),
    BoxedAccountSized4(BoxedAccountSized4),
    BoxedAccountSized8(BoxedAccountSized8),
    BoxedAccountUnsizedInit1(BoxedAccountUnsizedInit1),
    BoxedAccountUnsizedInit2(BoxedAccountUnsizedInit2),
    BoxedAccountUnsizedInit4(BoxedAccountUnsizedInit4),
    BoxedAccountUnsizedInit8(BoxedAccountUnsizedInit8),
    BoxedAccountUnsized1(BoxedAccountUnsized1),
    BoxedAccountUnsized2(BoxedAccountUnsized2),
    BoxedAccountUnsized4(BoxedAccountUnsized4),
    BoxedAccountUnsized8(BoxedAccountUnsized8),
    BoxedInterfaceAccountMint1(BoxedInterfaceAccountMint1),
    BoxedInterfaceAccountMint2(BoxedInterfaceAccountMint2),
    BoxedInterfaceAccountMint4(BoxedInterfaceAccountMint4),
    BoxedInterfaceAccountMint8(BoxedInterfaceAccountMint8),
    BoxedInterfaceAccountToken1(BoxedInterfaceAccountToken1),
    BoxedInterfaceAccountToken2(BoxedInterfaceAccountToken2),
    BoxedInterfaceAccountToken4(BoxedInterfaceAccountToken4),
    BoxedInterfaceAccountToken8(BoxedInterfaceAccountToken8),
    InterfaceAccountMint1(InterfaceAccountMint1),
    InterfaceAccountMint2(InterfaceAccountMint2),
    InterfaceAccountMint4(InterfaceAccountMint4),
    InterfaceAccountMint8(InterfaceAccountMint8),
    InterfaceAccountToken1(InterfaceAccountToken1),
    InterfaceAccountToken2(InterfaceAccountToken2),
    InterfaceAccountToken4(InterfaceAccountToken4),
    // Interface1(Interface1),
    // Interface2(Interface2),
    // Interface4(Interface4),
    // Interface8(Interface8),
    Program1(Program1),
    Program2(Program2),
    Program4(Program4),
    Program8(Program8),
    Signer1(Signer1),
    Signer2(Signer2),
    Signer4(Signer4),
    Signer8(Signer8),
    SystemAccount1(SystemAccount1),
    SystemAccount2(SystemAccount2),
    SystemAccount4(SystemAccount4),
    SystemAccount8(SystemAccount8),
    UncheckedAccount1(UncheckedAccount1),
    UncheckedAccount2(UncheckedAccount2),
    UncheckedAccount4(UncheckedAccount4),
    UncheckedAccount8(UncheckedAccount8),
}
use star_frame::borsh::{BorshDeserialize, BorshSerialize};

// Converted to StarFrame pattern: unit structs for args + AccountSet-suffixed account sets

// AccountInfo
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountInfo1;
#[derive(AccountSet, Debug)]
pub struct AccountInfo1Accounts {
    pub account1: AccountInfo,
}
empty_star_frame_instruction!(AccountInfo1, AccountInfo1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountInfo2;
#[derive(AccountSet, Debug)]
pub struct AccountInfo2Accounts {
    pub account1: AccountInfo,
    pub account2: AccountInfo,
}
empty_star_frame_instruction!(AccountInfo2, AccountInfo2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountInfo4;
#[derive(AccountSet, Debug)]
pub struct AccountInfo4Accounts {
    pub account1: AccountInfo,
    pub account2: AccountInfo,
    pub account3: AccountInfo,
    pub account4: AccountInfo,
}
empty_star_frame_instruction!(AccountInfo4, AccountInfo4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountInfo8;
#[derive(AccountSet, Debug)]
pub struct AccountInfo8Accounts {
    pub account1: AccountInfo,
    pub account2: AccountInfo,
    pub account3: AccountInfo,
    pub account4: AccountInfo,
    pub account5: AccountInfo,
    pub account6: AccountInfo,
    pub account7: AccountInfo,
    pub account8: AccountInfo,
}
empty_star_frame_instruction!(AccountInfo8, AccountInfo8Accounts);

#[derive(Copy, Clone, ProgramAccount, CheckedBitPattern, NoUninit, Align1, Zeroable, Debug)]
#[repr(C)]
pub struct Empty;

#[derive(Copy, Clone, ProgramAccount, CheckedBitPattern, NoUninit, Align1, Zeroable, Debug)]
#[repr(C)]
pub struct Sized {
    pub field: [u8; 8],
}

#[unsized_type(program_account)]
pub struct Unsized {
    #[unsized_start]
    pub field: List<u8>,
}

// (old Anchor-style AccountInfoX structs removed)

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountEmptyInit1;
#[derive(AccountSet, Debug)]
pub struct AccountEmptyInit1Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Empty>>>,
}
empty_star_frame_instruction!(AccountEmptyInit1, AccountEmptyInit1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountEmptyInit2;
#[derive(AccountSet, Debug)]
pub struct AccountEmptyInit2Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Empty>>>,
}
empty_star_frame_instruction!(AccountEmptyInit2, AccountEmptyInit2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountEmptyInit4;
#[derive(AccountSet, Debug)]
pub struct AccountEmptyInit4Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account3: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account4: Init<Signer<Account<Empty>>>,
}
empty_star_frame_instruction!(AccountEmptyInit4, AccountEmptyInit4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountEmptyInit8;
#[derive(AccountSet, Debug)]
pub struct AccountEmptyInit8Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account3: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account4: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account5: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account6: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account7: Init<Signer<Account<Empty>>>,
    #[validate(arg = Create(()))]
    pub account8: Init<Signer<Account<Empty>>>,
}
empty_star_frame_instruction!(AccountEmptyInit8, AccountEmptyInit8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountEmpty1;
#[derive(AccountSet, Debug)]
pub struct AccountEmpty1Accounts {
    pub account1: Account<Empty>,
}
empty_star_frame_instruction!(AccountEmpty1, AccountEmpty1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountEmpty2;
#[derive(AccountSet, Debug)]
pub struct AccountEmpty2Accounts {
    pub account1: Account<Empty>,
    pub account2: Account<Empty>,
}
empty_star_frame_instruction!(AccountEmpty2, AccountEmpty2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountEmpty4;
#[derive(AccountSet, Debug)]
pub struct AccountEmpty4Accounts {
    pub account1: Account<Empty>,
    pub account2: Account<Empty>,
    pub account3: Account<Empty>,
    pub account4: Account<Empty>,
}
empty_star_frame_instruction!(AccountEmpty4, AccountEmpty4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountEmpty8;
#[derive(AccountSet, Debug)]
pub struct AccountEmpty8Accounts {
    pub account1: Account<Empty>,
    pub account2: Account<Empty>,
    pub account3: Account<Empty>,
    pub account4: Account<Empty>,
    pub account5: Account<Empty>,
    pub account6: Account<Empty>,
    pub account7: Account<Empty>,
    pub account8: Account<Empty>,
}
empty_star_frame_instruction!(AccountEmpty8, AccountEmpty8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountSizedInit1;
#[derive(AccountSet, Debug)]
pub struct AccountSizedInit1Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Sized>>>,
}
empty_star_frame_instruction!(AccountSizedInit1, AccountSizedInit1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountSizedInit2;
#[derive(AccountSet, Debug)]
pub struct AccountSizedInit2Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Sized>>>,
}
empty_star_frame_instruction!(AccountSizedInit2, AccountSizedInit2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountSizedInit4;
#[derive(AccountSet, Debug)]
pub struct AccountSizedInit4Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account3: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account4: Init<Signer<Account<Sized>>>,
}
empty_star_frame_instruction!(AccountSizedInit4, AccountSizedInit4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountSizedInit8;
#[derive(AccountSet, Debug)]
pub struct AccountSizedInit8Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account3: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account4: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account5: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account6: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account7: Init<Signer<Account<Sized>>>,
    #[validate(arg = Create(()))]
    pub account8: Init<Signer<Account<Sized>>>,
}
empty_star_frame_instruction!(AccountSizedInit8, AccountSizedInit8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountSized1;
#[derive(AccountSet, Debug)]
pub struct AccountSized1Accounts {
    pub account1: Account<Sized>,
}
empty_star_frame_instruction!(AccountSized1, AccountSized1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountSized2;
#[derive(AccountSet, Debug)]
pub struct AccountSized2Accounts {
    pub account1: Account<Sized>,
    pub account2: Account<Sized>,
}
empty_star_frame_instruction!(AccountSized2, AccountSized2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountSized4;
#[derive(AccountSet, Debug)]
pub struct AccountSized4Accounts {
    pub account1: Account<Sized>,
    pub account2: Account<Sized>,
    pub account3: Account<Sized>,
    pub account4: Account<Sized>,
}
empty_star_frame_instruction!(AccountSized4, AccountSized4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountSized8;
#[derive(AccountSet, Debug)]
pub struct AccountSized8Accounts {
    pub account1: Account<Sized>,
    pub account2: Account<Sized>,
    pub account3: Account<Sized>,
    pub account4: Account<Sized>,
    pub account5: Account<Sized>,
    pub account6: Account<Sized>,
    pub account7: Account<Sized>,
    pub account8: Account<Sized>,
}
empty_star_frame_instruction!(AccountSized8, AccountSized8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountUnsizedInit1;
#[derive(AccountSet, Debug)]
pub struct AccountUnsizedInit1Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Unsized>>>,
}
empty_star_frame_instruction!(AccountUnsizedInit1, AccountUnsizedInit1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountUnsizedInit2;
#[derive(AccountSet, Debug)]
pub struct AccountUnsizedInit2Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Unsized>>>,
}
empty_star_frame_instruction!(AccountUnsizedInit2, AccountUnsizedInit2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountUnsizedInit4;
#[derive(AccountSet, Debug)]
pub struct AccountUnsizedInit4Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account3: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account4: Init<Signer<Account<Unsized>>>,
}
empty_star_frame_instruction!(AccountUnsizedInit4, AccountUnsizedInit4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountUnsizedInit8;
#[derive(AccountSet, Debug)]
pub struct AccountUnsizedInit8Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account2: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account3: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account4: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account5: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account6: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account7: Init<Signer<Account<Unsized>>>,
    #[validate(arg = Create(()))]
    pub account8: Init<Signer<Account<Unsized>>>,
}
empty_star_frame_instruction!(AccountUnsizedInit8, AccountUnsizedInit8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountUnsized1;
#[derive(AccountSet, Debug)]
pub struct AccountUnsized1Accounts {
    pub account1: Account<Unsized>,
}
empty_star_frame_instruction!(AccountUnsized1, AccountUnsized1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountUnsized2;
#[derive(AccountSet, Debug)]
pub struct AccountUnsized2Accounts {
    pub account1: Account<Unsized>,
    pub account2: Account<Unsized>,
}
empty_star_frame_instruction!(AccountUnsized2, AccountUnsized2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountUnsized4;
#[derive(AccountSet, Debug)]
pub struct AccountUnsized4Accounts {
    pub account1: Account<Unsized>,
    pub account2: Account<Unsized>,
    pub account3: Account<Unsized>,
    pub account4: Account<Unsized>,
}
empty_star_frame_instruction!(AccountUnsized4, AccountUnsized4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct AccountUnsized8;
#[derive(AccountSet, Debug)]
pub struct AccountUnsized8Accounts {
    pub account1: Account<Unsized>,
    pub account2: Account<Unsized>,
    pub account3: Account<Unsized>,
    pub account4: Account<Unsized>,
    pub account5: Account<Unsized>,
    pub account6: Account<Unsized>,
    pub account7: Account<Unsized>,
    pub account8: Account<Unsized>,
}
empty_star_frame_instruction!(AccountUnsized8, AccountUnsized8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountEmptyInit1;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountEmptyInit1Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Empty>>>>,
}
empty_star_frame_instruction!(BoxedAccountEmptyInit1, BoxedAccountEmptyInit1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountEmptyInit2;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountEmptyInit2Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account2: Box<Init<Signer<Account<Empty>>>>,
}
empty_star_frame_instruction!(BoxedAccountEmptyInit2, BoxedAccountEmptyInit2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountEmptyInit4;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountEmptyInit4Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account2: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account3: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account4: Box<Init<Signer<Account<Empty>>>>,
}
empty_star_frame_instruction!(BoxedAccountEmptyInit4, BoxedAccountEmptyInit4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountEmptyInit8;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountEmptyInit8Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account2: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account3: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account4: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account5: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account6: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account7: Box<Init<Signer<Account<Empty>>>>,
    #[validate(arg = Create(()))]
    pub account8: Box<Init<Signer<Account<Empty>>>>,
}
empty_star_frame_instruction!(BoxedAccountEmptyInit8, BoxedAccountEmptyInit8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountEmpty1;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountEmpty1Accounts {
    pub account1: Box<Account<Empty>>,
}
empty_star_frame_instruction!(BoxedAccountEmpty1, BoxedAccountEmpty1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountEmpty2;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountEmpty2Accounts {
    pub account1: Box<Account<Empty>>,
    pub account2: Box<Account<Empty>>,
}
empty_star_frame_instruction!(BoxedAccountEmpty2, BoxedAccountEmpty2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountEmpty4;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountEmpty4Accounts {
    pub account1: Box<Account<Empty>>,
    pub account2: Box<Account<Empty>>,
    pub account3: Box<Account<Empty>>,
    pub account4: Box<Account<Empty>>,
}
empty_star_frame_instruction!(BoxedAccountEmpty4, BoxedAccountEmpty4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountEmpty8;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountEmpty8Accounts {
    pub account1: Box<Account<Empty>>,
    pub account2: Box<Account<Empty>>,
    pub account3: Box<Account<Empty>>,
    pub account4: Box<Account<Empty>>,
    pub account5: Box<Account<Empty>>,
    pub account6: Box<Account<Empty>>,
    pub account7: Box<Account<Empty>>,
    pub account8: Box<Account<Empty>>,
}
empty_star_frame_instruction!(BoxedAccountEmpty8, BoxedAccountEmpty8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountSizedInit1;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountSizedInit1Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Sized>>>>,
}
empty_star_frame_instruction!(BoxedAccountSizedInit1, BoxedAccountSizedInit1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountSizedInit2;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountSizedInit2Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account2: Box<Init<Signer<Account<Sized>>>>,
}
empty_star_frame_instruction!(BoxedAccountSizedInit2, BoxedAccountSizedInit2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountSizedInit4;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountSizedInit4Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()) )]
    pub account2: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account3: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account4: Box<Init<Signer<Account<Sized>>>>,
}
empty_star_frame_instruction!(BoxedAccountSizedInit4, BoxedAccountSizedInit4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountSizedInit8;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountSizedInit8Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account2: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account3: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account4: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account5: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account6: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account7: Box<Init<Signer<Account<Sized>>>>,
    #[validate(arg = Create(()))]
    pub account8: Box<Init<Signer<Account<Sized>>>>,
}
empty_star_frame_instruction!(BoxedAccountSizedInit8, BoxedAccountSizedInit8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountSized1;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountSized1Accounts {
    pub account1: Box<Account<Sized>>,
}
empty_star_frame_instruction!(BoxedAccountSized1, BoxedAccountSized1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountSized2;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountSized2Accounts {
    pub account1: Box<Account<Sized>>,
    pub account2: Box<Account<Sized>>,
}
empty_star_frame_instruction!(BoxedAccountSized2, BoxedAccountSized2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountSized4;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountSized4Accounts {
    pub account1: Box<Account<Sized>>,
    pub account2: Box<Account<Sized>>,
    pub account3: Box<Account<Sized>>,
    pub account4: Box<Account<Sized>>,
}
empty_star_frame_instruction!(BoxedAccountSized4, BoxedAccountSized4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountSized8;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountSized8Accounts {
    pub account1: Box<Account<Sized>>,
    pub account2: Box<Account<Sized>>,
    pub account3: Box<Account<Sized>>,
    pub account4: Box<Account<Sized>>,
    pub account5: Box<Account<Sized>>,
    pub account6: Box<Account<Sized>>,
    pub account7: Box<Account<Sized>>,
    pub account8: Box<Account<Sized>>,
}
empty_star_frame_instruction!(BoxedAccountSized8, BoxedAccountSized8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountUnsizedInit1;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountUnsizedInit1Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Unsized>>>>,
}
empty_star_frame_instruction!(BoxedAccountUnsizedInit1, BoxedAccountUnsizedInit1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountUnsizedInit2;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountUnsizedInit2Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account2: Box<Init<Signer<Account<Unsized>>>>,
}
empty_star_frame_instruction!(BoxedAccountUnsizedInit2, BoxedAccountUnsizedInit2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountUnsizedInit4;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountUnsizedInit4Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account2: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account3: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account4: Box<Init<Signer<Account<Unsized>>>>,
}
empty_star_frame_instruction!(BoxedAccountUnsizedInit4, BoxedAccountUnsizedInit4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountUnsizedInit8;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountUnsizedInit8Accounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub system_program: Program<System>,
    #[validate(arg = Create(()))]
    pub account1: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account2: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account3: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account4: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account5: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account6: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account7: Box<Init<Signer<Account<Unsized>>>>,
    #[validate(arg = Create(()))]
    pub account8: Box<Init<Signer<Account<Unsized>>>>,
}
empty_star_frame_instruction!(BoxedAccountUnsizedInit8, BoxedAccountUnsizedInit8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountUnsized1;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountUnsized1Accounts {
    pub account1: Box<Account<Unsized>>,
}
empty_star_frame_instruction!(BoxedAccountUnsized1, BoxedAccountUnsized1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountUnsized2;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountUnsized2Accounts {
    pub account1: Box<Account<Unsized>>,
    pub account2: Box<Account<Unsized>>,
}
empty_star_frame_instruction!(BoxedAccountUnsized2, BoxedAccountUnsized2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountUnsized4;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountUnsized4Accounts {
    pub account1: Box<Account<Unsized>>,
    pub account2: Box<Account<Unsized>>,
    pub account3: Box<Account<Unsized>>,
    pub account4: Box<Account<Unsized>>,
}
empty_star_frame_instruction!(BoxedAccountUnsized4, BoxedAccountUnsized4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedAccountUnsized8;
#[derive(AccountSet, Debug)]
pub struct BoxedAccountUnsized8Accounts {
    pub account1: Box<Account<Unsized>>,
    pub account2: Box<Account<Unsized>>,
    pub account3: Box<Account<Unsized>>,
    pub account4: Box<Account<Unsized>>,
    pub account5: Box<Account<Unsized>>,
    pub account6: Box<Account<Unsized>>,
    pub account7: Box<Account<Unsized>>,
    pub account8: Box<Account<Unsized>>,
}
empty_star_frame_instruction!(BoxedAccountUnsized8, BoxedAccountUnsized8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedInterfaceAccountMint1;
#[derive(AccountSet, Debug)]
pub struct BoxedInterfaceAccountMint1Accounts {
    pub account1: Box<MintAccount>,
}
empty_star_frame_instruction!(
    BoxedInterfaceAccountMint1,
    BoxedInterfaceAccountMint1Accounts
);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedInterfaceAccountMint2;
#[derive(AccountSet, Debug)]
pub struct BoxedInterfaceAccountMint2Accounts {
    pub account1: Box<MintAccount>,
    pub account2: Box<MintAccount>,
}
empty_star_frame_instruction!(
    BoxedInterfaceAccountMint2,
    BoxedInterfaceAccountMint2Accounts
);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedInterfaceAccountMint4;
#[derive(AccountSet, Debug)]
pub struct BoxedInterfaceAccountMint4Accounts {
    pub account1: Box<MintAccount>,
    pub account2: Box<MintAccount>,
    pub account3: Box<MintAccount>,
    pub account4: Box<MintAccount>,
}
empty_star_frame_instruction!(
    BoxedInterfaceAccountMint4,
    BoxedInterfaceAccountMint4Accounts
);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedInterfaceAccountMint8;
#[derive(AccountSet, Debug)]
pub struct BoxedInterfaceAccountMint8Accounts {
    pub account1: Box<MintAccount>,
    pub account2: Box<MintAccount>,
    pub account3: Box<MintAccount>,
    pub account4: Box<MintAccount>,
    pub account5: Box<MintAccount>,
    pub account6: Box<MintAccount>,
    pub account7: Box<MintAccount>,
    pub account8: Box<MintAccount>,
}
empty_star_frame_instruction!(
    BoxedInterfaceAccountMint8,
    BoxedInterfaceAccountMint8Accounts
);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedInterfaceAccountToken1;
#[derive(AccountSet, Debug)]
pub struct BoxedInterfaceAccountToken1Accounts {
    pub account1: Box<TokenAccount>,
}
empty_star_frame_instruction!(
    BoxedInterfaceAccountToken1,
    BoxedInterfaceAccountToken1Accounts
);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedInterfaceAccountToken2;
#[derive(AccountSet, Debug)]
pub struct BoxedInterfaceAccountToken2Accounts {
    pub account1: Box<TokenAccount>,
    pub account2: Box<TokenAccount>,
}
empty_star_frame_instruction!(
    BoxedInterfaceAccountToken2,
    BoxedInterfaceAccountToken2Accounts
);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedInterfaceAccountToken4;
#[derive(AccountSet, Debug)]
pub struct BoxedInterfaceAccountToken4Accounts {
    pub account1: Box<TokenAccount>,
    pub account2: Box<TokenAccount>,
    pub account3: Box<TokenAccount>,
    pub account4: Box<TokenAccount>,
}
empty_star_frame_instruction!(
    BoxedInterfaceAccountToken4,
    BoxedInterfaceAccountToken4Accounts
);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct BoxedInterfaceAccountToken8;
#[derive(AccountSet, Debug)]
pub struct BoxedInterfaceAccountToken8Accounts {
    pub account1: Box<TokenAccount>,
    pub account2: Box<TokenAccount>,
    pub account3: Box<TokenAccount>,
    pub account4: Box<TokenAccount>,
    pub account5: Box<TokenAccount>,
    pub account6: Box<TokenAccount>,
    pub account7: Box<TokenAccount>,
    pub account8: Box<TokenAccount>,
}
empty_star_frame_instruction!(
    BoxedInterfaceAccountToken8,
    BoxedInterfaceAccountToken8Accounts
);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct InterfaceAccountMint1;
#[derive(AccountSet, Debug)]
pub struct InterfaceAccountMint1Accounts {
    pub account1: MintAccount,
}
empty_star_frame_instruction!(InterfaceAccountMint1, InterfaceAccountMint1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct InterfaceAccountMint2;
#[derive(AccountSet, Debug)]
pub struct InterfaceAccountMint2Accounts {
    pub account1: MintAccount,
    pub account2: MintAccount,
}
empty_star_frame_instruction!(InterfaceAccountMint2, InterfaceAccountMint2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct InterfaceAccountMint4;
#[derive(AccountSet, Debug)]
pub struct InterfaceAccountMint4Accounts {
    pub account1: MintAccount,
    pub account2: MintAccount,
    pub account3: MintAccount,
    pub account4: MintAccount,
}
empty_star_frame_instruction!(InterfaceAccountMint4, InterfaceAccountMint4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct InterfaceAccountMint8;
#[derive(AccountSet, Debug)]
pub struct InterfaceAccountMint8Accounts {
    pub account1: MintAccount,
    pub account2: MintAccount,
    pub account3: MintAccount,
    pub account4: MintAccount,
    pub account5: MintAccount,
    pub account6: MintAccount,
    pub account7: MintAccount,
    pub account8: MintAccount,
}
empty_star_frame_instruction!(InterfaceAccountMint8, InterfaceAccountMint8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct InterfaceAccountToken1;
#[derive(AccountSet, Debug)]
pub struct InterfaceAccountToken1Accounts {
    pub account1: TokenAccount,
}
empty_star_frame_instruction!(InterfaceAccountToken1, InterfaceAccountToken1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct InterfaceAccountToken2;
#[derive(AccountSet, Debug)]
pub struct InterfaceAccountToken2Accounts {
    pub account1: TokenAccount,
    pub account2: TokenAccount,
}
empty_star_frame_instruction!(InterfaceAccountToken2, InterfaceAccountToken2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct InterfaceAccountToken4;
#[derive(AccountSet, Debug)]
pub struct InterfaceAccountToken4Accounts {
    pub account1: TokenAccount,
    pub account2: TokenAccount,
    pub account3: TokenAccount,
    pub account4: TokenAccount,
}
empty_star_frame_instruction!(InterfaceAccountToken4, InterfaceAccountToken4Accounts);

/*
#[derive(Accounts)]
pub struct Interface1<'info> {
    pub account1: Interface<'info, TokenInterface>,
}
*/

/*
#[derive(Accounts)]
pub struct Interface2<'info> {
    pub account1: Interface<'info, TokenInterface>,
    pub account2: Interface<'info, TokenInterface>,
}
*/

/*
#[derive(Accounts)]
pub struct Interface4<'info> {
    pub account1: Interface<'info, TokenInterface>,
    pub account2: Interface<'info, TokenInterface>,
    pub account3: Interface<'info, TokenInterface>,
    pub account4: Interface<'info, TokenInterface>,
}
*/

/*
#[derive(Accounts)]
pub struct Interface8<'info> {
    pub account1: Interface<'info, TokenInterface>,
    pub account2: Interface<'info, TokenInterface>,
    pub account3: Interface<'info, TokenInterface>,
    pub account4: Interface<'info, TokenInterface>,
    pub account5: Interface<'info, TokenInterface>,
    pub account6: Interface<'info, TokenInterface>,
    pub account7: Interface<'info, TokenInterface>,
    pub account8: Interface<'info, TokenInterface>,
}
*/

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Program1;
#[derive(AccountSet, Debug)]
pub struct Program1Accounts {
    pub account1: Program<System>,
}
empty_star_frame_instruction!(Program1, Program1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Program2;
#[derive(AccountSet, Debug)]
pub struct Program2Accounts {
    pub account1: Program<System>,
    pub account2: Program<System>,
}
empty_star_frame_instruction!(Program2, Program2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Program4;
#[derive(AccountSet, Debug)]
pub struct Program4Accounts {
    pub account1: Program<System>,
    pub account2: Program<System>,
    pub account3: Program<System>,
    pub account4: Program<System>,
}
empty_star_frame_instruction!(Program4, Program4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Program8;
#[derive(AccountSet, Debug)]
pub struct Program8Accounts {
    pub account1: Program<System>,
    pub account2: Program<System>,
    pub account3: Program<System>,
    pub account4: Program<System>,
    pub account5: Program<System>,
    pub account6: Program<System>,
    pub account7: Program<System>,
    pub account8: Program<System>,
}
empty_star_frame_instruction!(Program8, Program8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Signer1;
#[derive(AccountSet, Debug)]
pub struct Signer1Accounts {
    pub account1: Signer<AccountInfo>,
}
empty_star_frame_instruction!(Signer1, Signer1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Signer2;
#[derive(AccountSet, Debug)]
pub struct Signer2Accounts {
    pub account1: Signer<AccountInfo>,
    pub account2: Signer<AccountInfo>,
}
empty_star_frame_instruction!(Signer2, Signer2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Signer4;
#[derive(AccountSet, Debug)]
pub struct Signer4Accounts {
    pub account1: Signer<AccountInfo>,
    pub account2: Signer<AccountInfo>,
    pub account3: Signer<AccountInfo>,
    pub account4: Signer<AccountInfo>,
}
empty_star_frame_instruction!(Signer4, Signer4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Signer8;
#[derive(AccountSet, Debug)]
pub struct Signer8Accounts {
    pub account1: Signer<AccountInfo>,
    pub account2: Signer<AccountInfo>,
    pub account3: Signer<AccountInfo>,
    pub account4: Signer<AccountInfo>,
    pub account5: Signer<AccountInfo>,
    pub account6: Signer<AccountInfo>,
    pub account7: Signer<AccountInfo>,
    pub account8: Signer<AccountInfo>,
}
empty_star_frame_instruction!(Signer8, Signer8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct SystemAccount1;
#[derive(AccountSet, Debug)]
pub struct SystemAccount1Accounts {
    pub account1: SystemAccount,
}
empty_star_frame_instruction!(SystemAccount1, SystemAccount1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct SystemAccount2;
#[derive(AccountSet, Debug)]
pub struct SystemAccount2Accounts {
    pub account1: SystemAccount,
    pub account2: SystemAccount,
}
empty_star_frame_instruction!(SystemAccount2, SystemAccount2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct SystemAccount4;
#[derive(AccountSet, Debug)]
pub struct SystemAccount4Accounts {
    pub account1: SystemAccount,
    pub account2: SystemAccount,
    pub account3: SystemAccount,
    pub account4: SystemAccount,
}
empty_star_frame_instruction!(SystemAccount4, SystemAccount4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct SystemAccount8;
#[derive(AccountSet, Debug)]
pub struct SystemAccount8Accounts {
    pub account1: SystemAccount,
    pub account2: SystemAccount,
    pub account3: SystemAccount,
    pub account4: SystemAccount,
    pub account5: SystemAccount,
    pub account6: SystemAccount,
    pub account7: SystemAccount,
    pub account8: SystemAccount,
}
empty_star_frame_instruction!(SystemAccount8, SystemAccount8Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct UncheckedAccount1;
#[derive(AccountSet, Debug)]
pub struct UncheckedAccount1Accounts {
    pub account1: AccountInfo,
}
empty_star_frame_instruction!(UncheckedAccount1, UncheckedAccount1Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct UncheckedAccount2;
#[derive(AccountSet, Debug)]
pub struct UncheckedAccount2Accounts {
    pub account1: AccountInfo,
    pub account2: AccountInfo,
}
empty_star_frame_instruction!(UncheckedAccount2, UncheckedAccount2Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct UncheckedAccount4;
#[derive(AccountSet, Debug)]
pub struct UncheckedAccount4Accounts {
    pub account1: AccountInfo,
    pub account2: AccountInfo,
    pub account3: AccountInfo,
    pub account4: AccountInfo,
}
empty_star_frame_instruction!(UncheckedAccount4, UncheckedAccount4Accounts);

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct UncheckedAccount8;
#[derive(AccountSet, Debug)]
pub struct UncheckedAccount8Accounts {
    pub account1: AccountInfo,
    pub account2: AccountInfo,
    pub account3: AccountInfo,
    pub account4: AccountInfo,
    pub account5: AccountInfo,
    pub account6: AccountInfo,
    pub account7: AccountInfo,
    pub account8: AccountInfo,
}
empty_star_frame_instruction!(UncheckedAccount8, UncheckedAccount8Accounts);
