pub mod macro_prelude {
    pub use crate::account_set::{
        AccountSet, CanInitAccount, CanSetSeeds, HasOwnerProgram, HasProgramAccount, HasSeeds,
        ProgramAccount, SignedAccount, SingleAccountSet, SingleAccountSetMetadata, WritableAccount,
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
        AccountSetToIdl, AccountToIdl, InstructionSetToIdl, InstructionToIdl, ProgramToIdl,
        TypeToIdl,
    };

    #[cfg(feature = "idl")]
    pub use star_frame_idl::{
        account::{IdlAccount, IdlAccountId},
        account_set::{IdlAccountSet, IdlAccountSetDef, IdlAccountSetId, IdlAccountSetStructField},
        instruction::{IdlInstruction, IdlInstructionDef},
        item_source,
        ty::{IdlEnumVariant, IdlStructField, IdlType, IdlTypeDef, IdlTypeId},
        IdlDefinition, IdlDefinitionReference, ItemInfo, Version,
    };
}
