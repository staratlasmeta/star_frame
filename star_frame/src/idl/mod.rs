use crate::prelude::*;
use star_frame_idl::account::IdlAccountId;
use star_frame_idl::account_set::IdlAccountSetDef;
use star_frame_idl::instruction::IdlInstructionDef;
use star_frame_idl::seeds::IdlSeeds;
use star_frame_idl::ty::IdlTypeDef;
use star_frame_idl::{CrateMetadata, IdlDefinition, IdlMetadata};
pub use star_frame_proc::{InstructionToIdl, TypeToIdl};

mod find_seeds;
mod ty;
pub use find_seeds::*;

pub trait InstructionSetToIdl: InstructionSet {
    /// Adds each instruction in an instruction set to the idl definition.
    fn instruction_set_to_idl(idl_definition: &mut IdlDefinition) -> Result<()>;
}
pub trait InstructionToIdl<A>: Instruction {
    /// Adds an instruction to the idl definition, handling any nested definitions as necessary.
    fn instruction_to_idl(idl_definition: &mut IdlDefinition, arg: A) -> Result<IdlInstructionDef>;
}
pub trait AccountSetToIdl<'info, A>: AccountSet<'info> {
    /// Adds the [`star_frame_idl::IdlAccountSetDef`] and associated account definitions to the idl definition.
    fn account_set_to_idl(idl_definition: &mut IdlDefinition, arg: A) -> Result<IdlAccountSetDef>;
}
pub trait AccountToIdl: TypeToIdl {
    /// Adds the [`star_frame_idl::IdlAccount`] and associated type definitions to the idl definition,
    /// returning the idl account id reference.
    fn account_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlAccountId>;
}

pub trait TypeToIdl {
    type AssociatedProgram: ProgramToIdl;
    /// Returns the idl of this type.
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef>;
}

pub trait SeedsToIdl: GetSeeds {
    /// Returns the [`IdlSeeds`] for a given [`GetSeeds`], adding any new types to the idl definition.
    fn seeds_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlSeeds>;
}

#[must_use]
pub fn empty_env_option(env: &str) -> Option<String> {
    if env.is_empty() {
        None
    } else {
        Some(env.to_string())
    }
}

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

pub trait ProgramToIdl: StarFrameProgram {
    #[must_use]
    fn crate_metadata() -> CrateMetadata {
        CrateMetadata {
            docs: vec!["Hello".into()],
            ..crate_metadata!()
        }
    }

    fn modify_idl(_idl_definition: &mut IdlDefinition) -> Result<()> {
        Ok(())
    }

    fn program_to_idl() -> Result<IdlDefinition>
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
        Self::modify_idl(&mut out)?;
        Ok(out)
    }
}
