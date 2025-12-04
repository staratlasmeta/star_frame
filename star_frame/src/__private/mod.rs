pub mod macro_prelude {
    #[cfg(not(target_os = "solana"))]
    pub use crate::account_set::ClientAccountSet;
    pub use crate::{
        account_set::{
            internal_reverse::{_account_set_cleanup_reverse, _account_set_validate_reverse},
            modifiers::{
                CanInitAccount, CanInitSeeds, GetSeeds, HasInnerType, HasOwnerProgram, HasSeeds,
                Seed, SignedAccount, WritableAccount,
            },
            single_set::{SingleAccountSet, SingleSetMeta},
            AccountSet, AccountSetValidate, CheckKey, CpiAccountSet, CpiConstWrapper,
            DynamicCpiAccountSetLen, ProgramAccount,
        },
        align1::Align1,
        bail,
        context::Context,
        cpi::{CpiBuilder, MakeCpi},
        errors::{ErrorCode, ErrorInfo, StarFrameError},
        instruction::{
            Instruction, InstructionArgs, InstructionDiscriminant, InstructionSet, IxArgs,
            IxReturnType, StarFrameInstruction,
        },
        program::StarFrameProgram,
        unsize::{
            init::{DefaultInit, UnsizedInit},
            wrapper::{
                ExclusiveRecurse, ExclusiveWrapper, SharedWrapper, StartPointer,
                UnsizedTypeDataAccess,
            },
            FromOwned, RawSliceAdvance, UnsizedType, UnsizedTypePtr,
        },
        Result,
    };

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    pub use crate::{
        crate_metadata,
        idl::{
            seed_const, seed_path, AccountSetToIdl, AccountToIdl, ErrorsToIdl, FindIdlSeeds,
            FindSeed, InstructionSetToIdl, InstructionToIdl, ProgramToIdl, SeedsToIdl, TypeToIdl,
        },
        IdlResult,
    };

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    pub use star_frame_idl::{
        account::{IdlAccount, IdlAccountId},
        account_set::{IdlAccountSet, IdlAccountSetDef, IdlAccountSetId, IdlAccountSetStructField},
        instruction::{IdlInstruction, IdlInstructionDef},
        item_source,
        seeds::{IdlFindSeed, IdlFindSeeds, IdlSeed, IdlSeeds},
        ty::{IdlEnumVariant, IdlStructField, IdlType, IdlTypeDef, IdlTypeId},
        CrateMetadata, ErrorNode, IdlDefinition, IdlDefinitionReference, ItemInfo, Version,
    };

    pub use star_frame_proc::{sighash, zero_copy, InstructionToIdl, TypeToIdl};

    pub use advancer::{Advance, AdvanceArray};
    pub use core::any::type_name;
    pub use derive_where::DeriveWhere;
    pub use pinocchio::{
        account::AccountView, error::ProgramError, instruction::InstructionAccount,
    };
    pub use solana_address::Address;
    #[cfg(not(target_os = "solana"))]
    pub use solana_instruction::AccountMeta;
    pub use solana_msg::msg;
    pub use typenum;
}
