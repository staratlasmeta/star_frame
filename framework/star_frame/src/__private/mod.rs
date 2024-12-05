pub mod macro_prelude {
    pub use crate::account_set::{
        internal_reverse::{_account_set_cleanup_reverse, _account_set_validate_reverse},
        AccountSet, AccountSetValidate, CanInitAccount, CanInitSeeds, GetSeeds, HasOwnerProgram,
        HasProgramAccount, HasSeeds, ProgramAccount, Seed, SignedAccount, SingleAccountSet,
        SingleSetMeta, WritableAccount,
    };
    pub use crate::align1::Align1;
    pub use crate::client::{
        ClientAccountSet, CpiAccountSet, CpiBuilder, MakeCpi, MakeInstruction,
    };
    pub use crate::instruction::{
        Instruction, InstructionDiscriminant, InstructionSet, StarFrameInstruction,
    };
    pub use crate::program::StarFrameProgram;
    pub use crate::sighash;
    pub use crate::syscalls::{SyscallAccountCache, SyscallInvoke};
    pub use crate::unsize::{
        AsBytes, AsMutBytes, FromBytesReturn, RefBytes, RefBytesMut, RefDeref, RefDerefMut,
        RefResize, RefWrapper, RefWrapperMutExt, RefWrapperTypes, Resize, UnsizedInit, UnsizedType,
        Zeroed, {CombinedExt, CombinedRef, CombinedUnsized, RefWrapperT, RefWrapperU},
    };
    pub use crate::Result;

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    pub use crate::idl::{
        seed_const, seed_path, AccountSetToIdl, AccountToIdl, FindIdlSeeds, FindSeed,
        InstructionSetToIdl, InstructionToIdl, ProgramToIdl, SeedsToIdl, TypeToIdl,
    };

    pub use solana_program::{account_info::AccountInfo, instruction::AccountMeta, pubkey::Pubkey};

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    pub use star_frame_idl::{
        account::{IdlAccount, IdlAccountId},
        account_set::{IdlAccountSet, IdlAccountSetDef, IdlAccountSetId, IdlAccountSetStructField},
        instruction::{IdlInstruction, IdlInstructionDef},
        item_source,
        seeds::{IdlFindSeed, IdlFindSeeds, IdlSeed, IdlSeeds},
        ty::{IdlEnumVariant, IdlStructField, IdlType, IdlTypeDef, IdlTypeId},
        CrateMetadata, IdlDefinition, IdlDefinitionReference, ItemInfo, Version,
    };
}
