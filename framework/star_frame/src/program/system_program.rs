use crate::program::{ProgramIds, StarFrameProgram};
use crate::sys_calls::SysCalls;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::SystemInstruction;
use solana_program::system_program;
use star_frame::instruction::InstructionSet;

pub struct SystemProgram;
impl StarFrameProgram for SystemProgram {
    type InstructionSet<'a> = SystemInstruction;
    type InstructionDiscriminant = ();
    type AccountDiscriminant = ();
    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = ();

    const PROGRAM_IDS: ProgramIds = ProgramIds::AllNetworks(&system_program::ID);
}
impl<'a> InstructionSet<'a> for SystemInstruction {
    type Discriminant = ();

    fn handle_ix(
        self,
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        panic!("System instruction should not be handled");
    }
}
#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use crate::account_set::mutable::Writable;
    use crate::account_set::signer::Signer;
    use crate::account_set::AccountSet;
    use crate::idl::ty::TypeToIdl;
    use crate::idl::{AccountSetToIdl, InstructionSetToIdl, ProgramToIdl};
    use solana_program::instruction::AccountMeta;
    use star_frame_idl::account_set::{
        AccountSetId, IdlAccountSet, IdlAccountSetDef, IdlAccountSetStructField,
    };
    use star_frame_idl::instruction::{IdlInstruction, IdlInstructionDef};
    use star_frame_idl::ty::{IdlField, IdlType, IdlTypeDef, TypeId};
    use star_frame_idl::{
        DiscriminantId, IdlDefinition, IdlDefinitionReference, NetworkKey, ProgramIds, SemVer,
        Version,
    };

    pub struct CreateAccountSet<'info> {
        pub funder: Writable<Signer<AccountInfo<'info>>>,
        pub new_account: Writable<Signer<AccountInfo<'info>>>,
    }
    impl<'info> AccountSet<'info> for CreateAccountSet<'info> {
        fn try_to_accounts<'a, E>(
            &'a self,
            mut add_account: impl FnMut(&'a AccountInfo<'info>) -> Result<(), E>,
        ) -> Result<(), E>
        where
            'info: 'a,
        {
            self.funder.try_to_accounts(&mut add_account)?;
            self.new_account.try_to_accounts(&mut add_account)?;
            Ok(())
        }

        fn to_account_metas(&self, mut add_account_meta: impl FnMut(AccountMeta)) {
            self.funder.to_account_metas(&mut add_account_meta);
            self.new_account.to_account_metas(&mut add_account_meta);
        }
    }
    impl<'info> AccountSetToIdl<'info, ()> for CreateAccountSet<'info> {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (),
        ) -> Result<IdlAccountSetDef> {
            let namespace = if idl_definition.namespace == SystemProgram::idl_namespace() {
                let funder = Writable::<Signer<AccountInfo<'info>>>::account_set_to_idl(
                    idl_definition,
                    arg,
                )?;
                let new_account = Writable::<Signer<AccountInfo<'info>>>::account_set_to_idl(
                    idl_definition,
                    arg,
                )?;
                idl_definition.account_sets.insert(
                    "CreateAccountSet".to_string(),
                    IdlAccountSet {
                        name: "Create Account Set".to_string(),
                        description: "Account Set for Create Account".to_string(),
                        type_generics: vec![],
                        account_generics: vec![],
                        def: IdlAccountSetDef::Struct(vec![
                            IdlAccountSetStructField {
                                name: "Funder".to_string(),
                                description: "Funding account".to_string(),
                                path: "funder".to_string(),
                                account_set: funder,
                                extension_fields: Default::default(),
                            },
                            IdlAccountSetStructField {
                                name: "New Account".to_string(),
                                description: "New account".to_string(),
                                path: "new_account".to_string(),
                                account_set: new_account,
                                extension_fields: Default::default(),
                            },
                        ]),
                        extension_fields: Default::default(),
                    },
                );
                None
            } else {
                idl_definition.required_idl_definitions.insert(
                    SystemProgram::idl_namespace().to_string(),
                    IdlDefinitionReference {
                        namespace: SystemProgram::idl_namespace().to_string(),
                        version: SemVer::Wildcard,
                    },
                );
                Some(SystemProgram::idl_namespace().to_string())
            };
            Ok(IdlAccountSetDef::AccountSet(AccountSetId {
                namespace,
                account_set_id: "CreateAccountSet".to_string(),
                provided_type_generics: vec![],
                provided_account_generics: vec![],
                extension_fields: Default::default(),
            }))
        }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct CreateAccountData {
        pub lamports: u64,
        pub space: u64,
        pub owner: Pubkey,
    }
    impl TypeToIdl for CreateAccountData {
        type AssociatedProgram = SystemProgram;

        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            let namespace = if idl_definition.namespace == Self::AssociatedProgram::idl_namespace()
            {
                let lamports = u64::type_to_idl(idl_definition)?;
                let space = u64::type_to_idl(idl_definition)?;
                let owner = Pubkey::type_to_idl(idl_definition)?;
                idl_definition.types.insert(
                    "CreateAccountData".to_string(),
                    IdlType {
                        name: "Create Account Data".to_string(),
                        description: "Data for CreateAccount".to_string(),
                        generics: vec![],
                        type_def: IdlTypeDef::Struct(vec![
                            IdlField {
                                name: "Lamports".to_string(),
                                description: "Number of lamports to transfer to the new account"
                                    .to_string(),
                                path_id: "lamports".to_string(),
                                type_def: lamports,
                                extension_fields: Default::default(),
                            },
                            IdlField {
                                name: "Space".to_string(),
                                description: "Number of bytes of memory to allocate".to_string(),
                                path_id: "space".to_string(),
                                type_def: space,
                                extension_fields: Default::default(),
                            },
                            IdlField {
                                name: "Owner".to_string(),
                                description: "Address of program that will own the new account"
                                    .to_string(),
                                path_id: "owner".to_string(),
                                type_def: owner,
                                extension_fields: Default::default(),
                            },
                        ]),
                        extension_fields: Default::default(),
                    },
                );
                None
            } else {
                idl_definition.required_idl_definitions.insert(
                    Self::AssociatedProgram::idl_namespace().to_string(),
                    IdlDefinitionReference {
                        namespace: Self::AssociatedProgram::idl_namespace().to_string(),
                        version: Self::type_program_versions(),
                    },
                );
                Some(Self::AssociatedProgram::idl_namespace().to_string())
            };
            Ok(IdlTypeDef::IdlType(TypeId {
                namespace,
                type_id: "CreateAccountData".to_string(),
                provided_generics: vec![],
                extension_fields: Default::default(),
            }))
        }

        fn type_program_versions() -> SemVer {
            SemVer::Wildcard
        }
    }

    impl<'a> InstructionSetToIdl<'a> for SystemInstruction {
        fn instruction_set_to_idl(idl_definition: &mut IdlDefinition) -> Result<()> {
            {
                let account_set = CreateAccountSet::account_set_to_idl(idl_definition, ())?;
                let data = CreateAccountData::type_to_idl(idl_definition)?;
                idl_definition.instructions.insert(
                    "CreateAccount".to_string(),
                    IdlInstruction {
                        name: "Create Account".to_string(),
                        description: "Create a new account".to_string(),
                        discriminant: Default::default(),
                        definition: IdlInstructionDef { account_set, data },
                        extension_fields: Default::default(),
                    },
                );
            }
            Ok(())
        }
    }
    impl ProgramToIdl for SystemProgram {
        const VERSION: Version = Version {
            major: 1,
            minor: 16,
            patch: 9,
        };

        fn program_to_idl() -> Result<IdlDefinition> {
            let mut out = IdlDefinition {
                idl_std_version: Version {
                    major: 0,
                    minor: 1,
                    patch: 0,
                },
                version: Self::VERSION,
                name: "System Program".to_string(),
                namespace: Self::idl_namespace().to_string(),
                description: "The Solana System Program".to_string(),
                required_plugins: Default::default(),
                required_idl_definitions: Default::default(),
                program_ids: ProgramIds::AllNetworks(NetworkKey {
                    key: system_program::id(),
                    extension_fields: Default::default(),
                }),
                account_discriminant: DiscriminantId::None,
                instruction_discriminant: DiscriminantId::U32,
                accounts: Default::default(),
                types: Default::default(),
                account_sets: Default::default(),
                instructions: Default::default(),
                extension_fields: Default::default(),
            };
            <SystemProgram as StarFrameProgram>::InstructionSet::instruction_set_to_idl(&mut out)?;
            Ok(out)
        }
        fn idl_namespace() -> &'static str {
            "@solana/system-program"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn print_idl() -> Result<(), String> {
            let idl = SystemProgram::program_to_idl().unwrap();
            assert!(idl.instructions.contains_key("CreateAccount"));
            assert!(idl.account_sets.contains_key("CreateAccountSet"));
            assert!(idl.types.contains_key("CreateAccountData"));
            let create_account_data = idl.types.get("CreateAccountData").unwrap();
            matches!(create_account_data.type_def, IdlTypeDef::Struct(_));

            // println!("{}", serde_json::to_string_pretty(&idl).unwrap());
            Ok(())
        }
    }
}
