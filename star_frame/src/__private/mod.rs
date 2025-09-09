pub mod macro_prelude {
    pub use crate::{
        account_set::{
            internal_reverse::{_account_set_cleanup_reverse, _account_set_validate_reverse},
            modifiers::{
                CanInitAccount, CanInitSeeds, GetSeeds, HasInnerType, HasOwnerProgram, HasSeeds,
                Seed, SignedAccount, WritableAccount,
            },
            single_set::{SingleAccountSet, SingleSetMeta},
            AccountSet, AccountSetValidate, CheckKey, ClientAccountSet, CpiAccountSet,
            CpiConstWrapper, DynamicCpiAccountSetLen, ProgramAccount,
        },
        align1::Align1,
        client::MakeInstruction,
        context::Context,
        cpi::{CpiBuilder, MakeCpi},
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
            AsShared, FromOwned, RawSliceAdvance, UnsizedType, UnsizedTypeMut,
        },
        Result,
    };

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    pub use crate::{
        crate_metadata,
        idl::{
            seed_const, seed_path, AccountSetToIdl, AccountToIdl, FindIdlSeeds, FindSeed,
            InstructionSetToIdl, InstructionToIdl, ProgramToIdl, SeedsToIdl, TypeToIdl,
        },
    };

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

    pub use star_frame_proc::{sighash, zero_copy, InstructionToIdl, TypeToIdl};

    pub use advancer::{Advance, AdvanceArray};
    pub use anyhow::{self, bail};
    pub use core::any::type_name;
    pub use derive_where::DeriveWhere;
    pub use pinocchio::{
        account_info::AccountInfo, instruction::AccountMeta as PinocchioAccountMeta, msg,
        pubkey::Pubkey as PinocchioPubkey,
    };
    pub use solana_instruction::{AccountMeta, Instruction as SolanaInstruction};
    pub use solana_pubkey::Pubkey;
    pub use typenum;
}
