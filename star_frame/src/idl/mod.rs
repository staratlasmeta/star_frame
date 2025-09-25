//! IDL generation for `star_frame` programs using [`star_frame_idl`].
use crate::{instruction::Instruction, prelude::*};
use star_frame_idl::{
    account::IdlAccountId, account_set::IdlAccountSetDef, instruction::IdlInstructionDef,
    seeds::IdlSeeds, ty::IdlTypeDef, CrateMetadata, IdlDefinition, IdlMetadata,
};

mod find_seeds;
mod ty;
pub use find_seeds::*;
pub(crate) use ty::*;

/// Derivable via [`derive@InstructionSet`].   
pub trait InstructionSetToIdl: InstructionSet {
    /// Adds each instruction in an instruction set to the idl definition.
    fn instruction_set_to_idl(idl_definition: &mut IdlDefinition) -> crate::IdlResult<()>;
}

/// Derivable via [`derive@InstructionToIdl`] or [`derive@InstructionArgs`].
pub trait InstructionToIdl<A>: Instruction {
    /// Adds an instruction to the idl definition, handling any nested definitions as necessary.
    fn instruction_to_idl(
        idl_definition: &mut IdlDefinition,
        arg: A,
    ) -> crate::IdlResult<IdlInstructionDef>;
}

/// Derivable via [`derive@AccountSet`].
pub trait AccountSetToIdl<A> {
    /// Adds the [`star_frame_idl::account_set::IdlAccountSetDef`] and associated account definitions to the idl definition.
    fn account_set_to_idl(
        idl_definition: &mut IdlDefinition,
        arg: A,
    ) -> crate::IdlResult<IdlAccountSetDef>;
}

/// Derivable via [`derive@ProgramAccount`].
pub trait AccountToIdl: TypeToIdl {
    /// Adds the [`star_frame_idl::account::IdlAccount`] and associated type definitions to the idl definition,
    /// returning the idl account id reference.
    fn account_to_idl(idl_definition: &mut IdlDefinition) -> crate::IdlResult<IdlAccountId>;
}

/// Derivable via [`derive@TypeToIdl`].
pub trait TypeToIdl {
    type AssociatedProgram: ProgramToIdl;
    /// Returns the idl of this type.
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> crate::IdlResult<IdlTypeDef>;
}

/// Derivable via [`derive@GetSeeds`].
pub trait SeedsToIdl: GetSeeds {
    /// Returns the [`IdlSeeds`] for a given [`GetSeeds`], adding any new types to the idl definition.
    fn seeds_to_idl(idl_definition: &mut IdlDefinition) -> crate::IdlResult<IdlSeeds>;
}

/// Derivable via [`star_frame_error`].
pub trait ErrorsToIdl {
    /// Adds the errors to the idl definition.
    fn errors_to_idl(idl_definition: &mut IdlDefinition) -> crate::IdlResult<()>;
}

impl ErrorsToIdl for () {
    fn errors_to_idl(_idl_definition: &mut IdlDefinition) -> crate::IdlResult<()> {
        Ok(())
    }
}

#[doc(hidden)]
#[must_use]
pub fn empty_env_option(env: &str) -> Option<String> {
    if env.is_empty() {
        None
    } else {
        Some(env.to_string())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! crate_metadata {
    () => {
        $crate::star_frame_idl::CrateMetadata {
            version: $crate::star_frame_idl::Version::parse(env!("CARGO_PKG_VERSION"))
                .expect("Invalid package version. This should never happen."),
            name: env!("CARGO_PKG_NAME").to_string(),
            description: $crate::idl::empty_env_option(env!("CARGO_PKG_DESCRIPTION")),
            docs: vec![],
            homepage: $crate::idl::empty_env_option(env!("CARGO_PKG_HOMEPAGE")),
            license: $crate::idl::empty_env_option(env!("CARGO_PKG_LICENSE")),
            repository: $crate::idl::empty_env_option(env!("CARGO_PKG_REPOSITORY")),
        }
    };
}

/// The root IDL generation trait to generate an [`IdlDefinition`] for a program.
///
/// This should be derived via [`derive@StarFrameProgram`].
pub trait ProgramToIdl: StarFrameProgram {
    type Errors: ErrorsToIdl;
    #[must_use]
    fn crate_metadata() -> CrateMetadata {
        CrateMetadata {
            ..crate_metadata!()
        }
    }

    fn modify_idl(_idl_definition: &mut IdlDefinition) -> crate::IdlResult<()> {
        Ok(())
    }

    fn program_to_idl() -> crate::IdlResult<IdlDefinition>
    where
        <Self as StarFrameProgram>::InstructionSet: InstructionSetToIdl,
    {
        let mut out = IdlDefinition {
            address: Self::ID,
            metadata: IdlMetadata {
                crate_metadata: Self::crate_metadata(),
                ..Default::default()
            },
            ..Default::default()
        };
        <Self as StarFrameProgram>::InstructionSet::instruction_set_to_idl(&mut out)?;
        Self::Errors::errors_to_idl(&mut out)?;
        Self::modify_idl(&mut out)?;
        Ok(out)
    }
}
