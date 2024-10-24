use crate::prelude::*;
use solana_program::system_instruction::SystemInstruction;
use solana_program::system_program;

/// Solana's system program.
#[derive(Debug, Copy, Clone, Align1)]
pub struct SystemProgram;
impl StarFrameProgram for SystemProgram {
    type InstructionSet = SystemInstruction;
    type AccountDiscriminant = ();
    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = ();
    const PROGRAM_ID: Pubkey = system_program::ID;
}

impl InstructionSet for SystemInstruction {
    type Discriminant = ();

    fn handle_ix<'info>(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo<'info>],
        _ix_bytes: &[u8],
        _syscalls: &mut impl Syscalls<'info>,
    ) -> Result<()> {
        panic!("System instruction should not be handled");
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use crate::account_set::AccountSet;
    use crate::idl::TypeToIdl;
    use crate::idl::{AccountSetToIdl, InstructionSetToIdl, ProgramToIdl};
    use solana_program::instruction::AccountMeta;
    use star_frame_idl::account_set::{
        IdlAccountSet, IdlAccountSetDef, IdlAccountSetId, IdlAccountSetStructField,
    };
    use star_frame_idl::instruction::IdlInstructionDef;
    use star_frame_idl::ty::{IdlStructField, IdlType, IdlTypeDef, IdlTypeId};
    use star_frame_idl::{item_source, IdlDefinition, ItemInfo, Version};

    // todo: macroify all this.

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
            let source = item_source::<Self>();
            let funder =
                Writable::<Signer<AccountInfo<'info>>>::account_set_to_idl(idl_definition, arg)?;
            let new_account =
                Writable::<Signer<AccountInfo<'info>>>::account_set_to_idl(idl_definition, arg)?;
            let account_set = IdlAccountSet {
                info: ItemInfo {
                    name: "Create Account Set".to_string(),
                    description: vec![],
                    source: source.clone(),
                },
                type_generics: vec![],
                account_generics: vec![],
                account_set_def: IdlAccountSetDef::Struct(vec![
                    IdlAccountSetStructField {
                        description: vec!["Funder of the new account".to_string()],
                        path: "funder".to_string(),
                        account_set_def: funder,
                    },
                    IdlAccountSetStructField {
                        description: vec!["New account to create".to_string()],
                        path: "new_account".to_string(),
                        account_set_def: new_account,
                    },
                ]),
            };
            idl_definition.add_account_set(account_set);
            Ok(IdlAccountSetDef::Defined(IdlAccountSetId {
                source,
                provided_type_generics: vec![],
                provided_account_generics: vec![],
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
            let source = item_source::<Self>();
            // let namespace = if idl_definition.namespace == Self::AssociatedProgram::idl_namespace()
            // {
            let lamports = u64::type_to_idl(idl_definition)?;
            let space = u64::type_to_idl(idl_definition)?;
            let owner = Pubkey::type_to_idl(idl_definition)?;
            let idl_type = IdlType {
                info: ItemInfo {
                    name: "Create Account Data".to_string(),
                    description: vec![],
                    source: source.clone(),
                },
                generics: vec![],
                type_def: IdlTypeDef::Struct(vec![
                    IdlStructField {
                        description: vec![
                            "Number of lamports to transfer to the new account".to_string()
                        ],
                        path: Some("lamports".into()),
                        type_def: lamports,
                    },
                    IdlStructField {
                        description: vec!["Number of bytes of memory to allocate".into()],
                        path: Some("space".into()),
                        type_def: space,
                    },
                    IdlStructField {
                        description: vec!["Address of program that will own the new account".into()],
                        path: Some("owner".into()),
                        type_def: owner,
                    },
                ]),
            };
            let namespace = idl_definition.add_type(idl_type, SystemProgram::PROGRAM_ID);
            Ok(IdlTypeDef::Defined(IdlTypeId {
                namespace,
                source,
                provided_generics: vec![],
            }))
        }
    }

    impl InstructionSetToIdl for SystemInstruction {
        fn instruction_set_to_idl(idl_definition: &mut IdlDefinition) -> Result<()> {
            let account_set = CreateAccountSet::account_set_to_idl(idl_definition, ())?;
            let data = CreateAccountData::type_to_idl(idl_definition)?;
            let def = IdlInstructionDef {
                account_set,
                definition: data,
            };
            idl_definition.add_instruction(def, Default::default())?;
            Ok(())
        }
    }

    impl ProgramToIdl for SystemProgram {
        fn version() -> Version {
            Version::new(1, 18, 10)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn print_idl() {
            let idl = SystemProgram::program_to_idl().unwrap();
            // todo: more asserts
            // assert!(idl.instructions.contains_key("CreateAccount"));
            // assert!(idl.account_sets.contains_key("CreateAccountSet"));
            // assert!(idl.types.contains_key("CreateAccountData"));
            // let create_account_data = idl.types.get("CreateAccountData").unwrap();
            // matches!(create_account_data.type_def, IdlTypeDef::Struct(_));

            println!("{}", serde_json::to_string_pretty(&idl).unwrap());
        }
    }
}
