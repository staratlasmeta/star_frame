pub mod macro_prelude {
    pub use crate::account_set::{
        AccountSet, CanInitAccount, CanSetSeeds, GetSeeds, HasOwnerProgram, HasProgramAccount,
        HasSeeds, ProgramAccount, Seed, SignedAccount, SingleAccountSet, WritableAccount,
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

    #[cfg(feature = "idl")]
    pub use crate::idl::{
        seed_const, seed_path, AccountSetToIdl, AccountToIdl, FindIdlSeeds, FindSeed,
        InstructionSetToIdl, InstructionToIdl, ProgramToIdl, SeedsToIdl, TypeToIdl,
    };

    #[cfg(feature = "idl")]
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
