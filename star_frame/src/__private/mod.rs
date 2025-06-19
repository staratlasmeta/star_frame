pub mod macro_prelude {
    pub use crate::account_set::{
        internal_reverse::{_account_set_cleanup_reverse, _account_set_validate_reverse},
        AccountSet, AccountSetValidate, CanInitAccount, CanInitSeeds, GetSeeds, HasInnerType,
        HasOwnerProgram, HasSeeds, ProgramAccount, Seed, SignedAccount, SingleAccountSet,
        SingleSetMeta, WritableAccount,
    };
    pub use crate::align1::Align1;
    pub use crate::client::{
        ClientAccountSet, CpiAccountSet, CpiBuilder, MakeCpi, MakeInstruction,
    };
    pub use crate::instruction::{
        Instruction, InstructionArgs, InstructionDiscriminant, InstructionSet, IxArgs,
        StarFrameInstruction,
    };
    pub use crate::program::StarFrameProgram;

    pub use crate::context::Context;
    pub use crate::unsize::{
        init::{DefaultInit, UnsizedInit},
        wrapper::{
            ExclusiveRecurse, ExclusiveWrapper, LengthTracker, SharedWrapper, UnsizedTypeDataAccess,
        },
        AsShared, FromOwned, RawSliceAdvance, UnsizedType,
    };

    pub use crate::Result;
    pub use star_frame_proc::{sighash, InstructionToIdl, TypeToIdl};

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    pub use crate::{
        crate_metadata,
        idl::{
            seed_const, seed_path, AccountSetToIdl, AccountToIdl, FindIdlSeeds, FindSeed,
            InstructionSetToIdl, InstructionToIdl, ProgramToIdl, SeedsToIdl, TypeToIdl,
        },
    };

    pub use advancer::{Advance, AdvanceArray};
    pub use anyhow::{self, bail};

    pub use core::any::type_name;

    pub use pinocchio::{account_info::AccountInfo, msg, pubkey::Pubkey as PinocchioPubkey};
    pub use solana_instruction::{AccountMeta, Instruction as SolanaInstruction};
    pub use solana_pubkey::Pubkey;

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

    pub use derive_where::DeriveWhere;
}
