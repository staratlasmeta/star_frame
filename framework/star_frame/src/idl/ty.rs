use crate::data_types::PodBool;
use crate::idl::ProgramToIdl;
use crate::program::system_program::SystemProgram;
use crate::Result;
use solana_program::pubkey::Pubkey;
use star_frame_idl::ty::{IdlDefinedType, IdlType, IdlTypeDef, TypeId};
use star_frame_idl::{IdlDefinition, IdlDefinitionReference, SemVer};
pub use star_frame_proc::TypeToIdl;

pub trait TypeToIdl {
    type AssociatedProgram: ProgramToIdl;
    /// Returns the idl of this type.
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef>;
    #[must_use]
    fn type_program_versions() -> SemVer {
        SemVer::from_version(Self::AssociatedProgram::VERSION)
    }
}

macro_rules! impl_type_to_idl_for_defined {
    (@impl $ty:ty: $ident:ident) => {
        impl TypeToIdl for $ty {
            type AssociatedProgram = SystemProgram;

            fn type_to_idl(_idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
                Ok(IdlTypeDef::Defined(IdlDefinedType::$ident))
            }

            fn type_program_versions() -> SemVer {
                SemVer::Wildcard
            }
        }
    };
    ($($ty:ty: $ident:ident),* $(,)?) => {
        $(impl_type_to_idl_for_defined!(@impl $ty: $ident);)*
    };
}
impl_type_to_idl_for_defined!(
    u8: U8,
    u16: U16,
    u32: U32,
    u64: U64,
    u128: U128,
    i8: I8,
    i16: I16,
    i32: I32,
    i64: I64,
    i128: I128,
    bool: BorshBool,
    String: BorshString,
    PodBool: PodBool,
    // OptionalPubkey: OptionalPubkey,
);

impl TypeToIdl for Pubkey {
    type AssociatedProgram = SystemProgram;

    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        let namespace = if idl_definition.namespace == Self::AssociatedProgram::idl_namespace() {
            let u8 = u8::type_to_idl(idl_definition)?;
            idl_definition.add_type_if_missing("Pubkey", || IdlType {
                name: "Pubkey".to_string(),
                description: "A Solana public key".to_string(),
                generics: vec![],
                type_def: IdlTypeDef::Array {
                    item_ty: Box::new(u8),
                    size: 32,
                },
                extension_fields: Default::default(),
            });
            None
        } else {
            idl_definition.required_idl_definitions.insert(
                Self::AssociatedProgram::idl_namespace().to_string(),
                IdlDefinitionReference {
                    namespace: Self::AssociatedProgram::idl_namespace().to_string(),
                    version: SemVer::Wildcard,
                },
            );
            Some(Self::AssociatedProgram::idl_namespace().to_string())
        };
        Ok(IdlTypeDef::IdlType(TypeId {
            namespace,
            type_id: "Pubkey".to_string(),
            provided_generics: vec![],
            extension_fields: Default::default(),
        }))
    }
}

impl<T: TypeToIdl> TypeToIdl for Option<T> {
    type AssociatedProgram = SystemProgram;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        Ok(IdlTypeDef::BorshOption {
            item_ty: Box::new(T::type_to_idl(idl_definition)?),
        })
    }
}

impl<T: TypeToIdl> TypeToIdl for Vec<T> {
    type AssociatedProgram = SystemProgram;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        Ok(IdlTypeDef::BorshVec {
            item_ty: Box::new(T::type_to_idl(idl_definition)?),
            len_ty: Box::new(u32::type_to_idl(idl_definition)?),
        })
    }
}

// impl<T, L> TypeToIdl for List<T, L>
// where
//     T: CheckedBitPattern + NoUninit + TypeToIdl + Align1,
//     L: Pod + TypeToIdl + FromPrimitive + ToPrimitive,
// {
//     type AssociatedProgram = SystemProgram;
//     fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
//         Ok(IdlTypeDef::BorshVec {
//             item_ty: Box::new(T::type_to_idl(idl_definition)?),
//             len_ty: Box::new(L::type_to_idl(idl_definition)?),
//         })
//     }
// }

impl<T: TypeToIdl, const N: usize> TypeToIdl for [T; N] {
    type AssociatedProgram = SystemProgram;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        Ok(IdlTypeDef::Array {
            item_ty: Box::new(T::type_to_idl(idl_definition)?),
            size: N,
        })
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    enum InnerType {
        Hello1,
        Hello2,
    }

    // TODO: Generics? Should we handle this now?
    // TODO: lifetimes?
    // #[derive(IdlType)]
    // struct GenericStuff<T, U> {
    //     hello: T,
    //     stuff: Vec<U>,
    // }

    #[derive(TypeToIdl)]
    struct OtherStuff {
        string: String,
    }

    /// Description here too!
    #[derive(TypeToIdl)]
    struct ExampleStruct {
        /// This is a description of pubkey
        pubkey: Pubkey,
        vec_stuff: Vec<Option<OtherStuff>>,
        hello: u8,
        stuff: [u8; 32],
    }

    use super::*;
    use star_frame_idl::DiscriminantId;

    #[test]
    fn print_idl() -> Result<()> {
        let mut idl_definition = IdlDefinition {
            namespace: "my_program".to_string(),
            program_id: Pubkey::default(),
            account_discriminant: DiscriminantId::None,
            instruction_discriminant: DiscriminantId::None,
            idl_std_version: Default::default(),
            version: Default::default(),
            name: "".to_string(),
            description: "".to_string(),
            required_plugins: Default::default(),
            required_idl_definitions: Default::default(),
            accounts: Default::default(),
            types: Default::default(),
            account_sets: Default::default(),
            instructions: Default::default(),
            extension_fields: Default::default(),
        };
        ExampleStruct::type_to_idl(&mut idl_definition)?;
        Ok(())
    }
}
