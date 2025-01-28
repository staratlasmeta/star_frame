use crate::account::IdlAccount;
use crate::ty::{IdlEnumVariant, IdlStructField, IdlType, IdlTypeDef};
use crate::{IdlDefinition, IdlMetadata, ItemDescription, ItemInfo};
use anchor_lang_idl_spec as anchor;
use anyhow::{bail, Context, Result};

use crate::account_set::{IdlAccountSetDef, IdlSingleAccountSet};
use crate::instruction::IdlInstruction;
use crate::seeds::{IdlFindSeed, IdlFindSeeds};
pub use anchor::Idl as AnchorIdl;

impl TryFrom<IdlDefinition> for AnchorIdl {
    type Error = anyhow::Error;
    fn try_from(idl_def: IdlDefinition) -> Result<Self, Self::Error> {
        let instructions = idl_def
            .instructions
            .values()
            .map(|i| i.try_to_anchor(&idl_def))
            .collect::<Result<Vec<_>>>()?;
        let accounts = idl_def
            .accounts
            .values()
            .map(|a| a.try_to_anchor(&idl_def))
            .collect::<Result<Vec<_>>>()?;
        let types = idl_def
            .types
            .values()
            .chain(idl_def.external_types.values())
            .map(|t| t.try_to_anchor(&idl_def))
            .collect::<Result<Vec<_>>>()?;
        let idl = anchor::Idl {
            address: idl_def.address.to_string(),
            metadata: idl_def.metadata.try_to_anchor(&idl_def)?,
            instructions,
            accounts,
            types,
            docs: idl_def.metadata.crate_metadata.docs,
            // unsupported
            events: vec![],
            errors: vec![],
            constants: vec![],
        };
        Ok(idl)
    }
}

trait TryToAnchor<Output> {
    fn try_to_anchor(&self, idl_definition: &IdlDefinition) -> Result<Output>;
}

impl TryToAnchor<anchor::IdlMetadata> for IdlMetadata {
    fn try_to_anchor(
        &self,
        _idl_definition: &IdlDefinition,
    ) -> Result<anchor_lang_idl_spec::IdlMetadata> {
        let crate_metadata = self.crate_metadata.clone();
        Ok(anchor::IdlMetadata {
            spec: anchor_lang_idl_spec::IDL_SPEC.to_string(),
            name: crate_metadata.name,
            version: crate_metadata.version.to_string(),
            description: crate_metadata.description,
            repository: crate_metadata.repository,
            dependencies: vec![],
            contact: None,
            deployments: None,
        })
    }
}

impl TryToAnchor<anchor::IdlInstruction> for IdlInstruction {
    fn try_to_anchor(&self, idl_definition: &IdlDefinition) -> Result<anchor::IdlInstruction> {
        let ty = self.definition.type_id.get_defined(idl_definition)?;
        let args = match &ty.type_def {
            IdlTypeDef::Struct(fields) => 'matchy: {
                let Some(first_field) = fields.first() else {
                    break 'matchy Ok(vec![]);
                };
                if first_field.path.is_none() {
                    // tuple struct => single field with arg: DefinedType
                    Ok(vec![anchor::IdlField {
                        docs: ty.info.description.clone(),
                        name: "arg".to_string(),
                        ty: ty.type_def.try_to_anchor(idl_definition)?,
                    }])
                } else {
                    fields
                        .iter()
                        .map(|field| {
                            Ok(anchor::IdlField {
                                name: field.path.clone().with_context(|| {
                                    format!("Missing field name on named struct: {fields:#?}")
                                })?,
                                docs: field.description.clone(),
                                ty: field.type_def.try_to_anchor(idl_definition)?,
                            })
                        })
                        .collect::<Result<Vec<_>>>()
                }
            }
            type_def => Ok(vec![anchor::IdlField {
                docs: ty.info.description.clone(),
                name: "arg".to_string(),
                ty: type_def.try_to_anchor(idl_definition)?,
            }]),
        }?;
        let accounts = match self.definition.account_set.try_to_anchor(idl_definition)? {
            anchor::IdlInstructionAccountItem::Composite(composite) => composite.accounts,
            single => vec![single],
        };
        let ix = anchor::IdlInstruction {
            name: ty.info.name.clone(),
            docs: ty.info.description.clone(),
            discriminator: self.discriminant.clone(),
            accounts,
            args,
            returns: None,
        };
        Ok(ix)
    }
}

impl TryToAnchor<anchor::IdlInstructionAccountItem> for IdlAccountSetDef {
    fn try_to_anchor(
        &self,
        idl_definition: &IdlDefinition,
    ) -> Result<anchor::IdlInstructionAccountItem> {
        let mut incompat_set: Option<(String, IdlAccountSetDef)> = None;

        let defined = self.get_defined(idl_definition)?;
        account_set_to_anchor_inner(
            &defined.account_set_def,
            &defined.info,
            idl_definition,
            &mut incompat_set,
        )
    }
}

fn account_set_to_anchor_inner(
    set: &IdlAccountSetDef,
    info: &ItemInfo,
    idl_definition: &IdlDefinition,
    incompat_set: &mut Option<(String, IdlAccountSetDef)>,
) -> Result<anchor::IdlInstructionAccountItem> {
    if let Some((path, incompat_set)) = incompat_set {
        bail!(
            "Incompatible account sets must be the last item in an account set. Non-last set found at `{path}. {incompat_set:?}`"
        );
    }
    match &set {
        IdlAccountSetDef::Single(single_set) => Ok(anchor::IdlInstructionAccountItem::Single(
            single_account_set_to_anchor(single_set, &info.name, &info.description),
        )),
        IdlAccountSetDef::Struct(fields) => {
            let accounts = fields
                .iter()
                .enumerate()
                .map(|(index, f)| {
                    let name = f.path.clone().unwrap_or_else(|| index.to_string());
                    account_set_to_anchor_inner(
                        &f.account_set_def,
                        &ItemInfo {
                            name: name.clone(),
                            description: f.description.clone(),
                            source: format!("{}.{name}", info.source),
                        },
                        idl_definition,
                        incompat_set,
                    )
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter(|item| {
                    // filter out empty composite accounts
                    !matches!(item, anchor::IdlInstructionAccountItem::Composite(
                            anchor::IdlInstructionAccounts { accounts, .. }) if accounts.is_empty())
                })
                .collect();

            Ok(anchor::IdlInstructionAccountItem::Composite(
                anchor::IdlInstructionAccounts {
                    accounts,
                    name: info.name.to_string(),
                },
            ))
        }
        IdlAccountSetDef::Defined(_) => {
            let defined = set.get_defined(idl_definition)?;
            account_set_to_anchor_inner(
                &defined.account_set_def,
                &defined.info,
                idl_definition,
                incompat_set,
            )
        }
        IdlAccountSetDef::Many { .. } | IdlAccountSetDef::Or(_) => {
            eprintln!("Incompatible account set found at `{}`", info.source);
            incompat_set.replace((info.source.to_string(), set.clone()));
            Ok(anchor::IdlInstructionAccountItem::Composite(
                anchor::IdlInstructionAccounts {
                    accounts: vec![],
                    name: info.name.to_string(),
                },
            ))
        }
    }
}

impl TryToAnchor<anchor::IdlAccount> for IdlAccount {
    fn try_to_anchor(&self, idl_definition: &IdlDefinition) -> Result<anchor::IdlAccount> {
        let ty = self.type_id.get_defined(idl_definition)?;
        let account = anchor::IdlAccount {
            name: ty.info.name.clone(),
            discriminator: self.discriminant.clone(),
        };
        Ok(account)
    }
}

impl TryToAnchor<anchor::IdlTypeDef> for IdlType {
    fn try_to_anchor(&self, idl_definition: &IdlDefinition) -> Result<anchor::IdlTypeDef> {
        let ty = self.type_def.try_to_anchor(idl_definition)?;
        Ok(anchor::IdlTypeDef {
            name: self.info.name.clone(),
            docs: self.info.description.clone(),
            serialization: Default::default(),
            repr: None,
            generics: vec![],
            ty,
        })
    }
}

impl TryToAnchor<anchor::IdlTypeDefTy> for IdlTypeDef {
    fn try_to_anchor(&self, idl_definition: &IdlDefinition) -> Result<anchor::IdlTypeDefTy> {
        macro_rules! ty {
            ($ty:expr) => {
                anchor::IdlTypeDefTy::Type { alias: $ty }
            };
        }
        let type_def = match self {
            IdlTypeDef::Defined(type_id) => {
                let ty = idl_definition
                    .get_type(&type_id.source)
                    .context("Type not found")?;
                ty!(anchor::IdlType::Defined {
                    name: ty.info.name.clone(),
                    // todo: maybe support generics?
                    generics: vec![],
                })
            }
            IdlTypeDef::Generic(s) => ty!(anchor::IdlType::Generic(s.clone())),
            IdlTypeDef::Bool => ty!(anchor::IdlType::Bool),
            IdlTypeDef::U8 => ty!(anchor::IdlType::U8),
            IdlTypeDef::I8 => ty!(anchor::IdlType::I8),
            IdlTypeDef::U16 => ty!(anchor::IdlType::U16),
            IdlTypeDef::I16 => ty!(anchor::IdlType::I16),
            IdlTypeDef::U32 => ty!(anchor::IdlType::U32),
            IdlTypeDef::I32 => ty!(anchor::IdlType::I32),
            IdlTypeDef::F32 => ty!(anchor::IdlType::F32),
            IdlTypeDef::U64 => ty!(anchor::IdlType::U64),
            IdlTypeDef::I64 => ty!(anchor::IdlType::I64),
            IdlTypeDef::F64 => ty!(anchor::IdlType::F64),
            IdlTypeDef::U128 => ty!(anchor::IdlType::U128),
            IdlTypeDef::I128 => ty!(anchor::IdlType::I128),
            IdlTypeDef::String => ty!(anchor::IdlType::String),
            IdlTypeDef::Pubkey => ty!(anchor::IdlType::Pubkey),
            IdlTypeDef::FixedPoint { ty, .. } => ty
                .try_to_anchor(idl_definition)
                .context("Fixed point must be anchor compatible")?,
            IdlTypeDef::Option(ty) => ty!(anchor::IdlType::Option(Box::new(
                ty.try_to_anchor(idl_definition)?
            ))),
            IdlTypeDef::List { item_ty, len_ty } => {
                if **len_ty != IdlTypeDef::U32 {
                    bail!("Unsupported list length type")
                }
                ty!(anchor::IdlType::Vec(Box::new(
                    item_ty.try_to_anchor(idl_definition)?
                )))
            }
            IdlTypeDef::Array(ty, len) => ty!(anchor::IdlType::Array(
                Box::new(ty.try_to_anchor(idl_definition)?),
                anchor::IdlArrayLen::Value(*len)
            )),
            IdlTypeDef::Struct(fields) => {
                let fields = convert_fields_to_anchor(fields, idl_definition)?;
                anchor::IdlTypeDefTy::Struct { fields }
            }
            IdlTypeDef::Enum(variants) => {
                let variants = convert_variants_to_anchor(variants, idl_definition)?;
                anchor::IdlTypeDefTy::Enum { variants }
            }
        };
        Ok(type_def)
    }
}

impl TryToAnchor<anchor::IdlType> for IdlTypeDef {
    fn try_to_anchor(&self, idl_definition: &IdlDefinition) -> Result<anchor::IdlType> {
        if let Ok(anchor::IdlTypeDefTy::Type { alias }) = self.try_to_anchor(idl_definition) {
            Ok(alias)
        } else {
            bail!("Unsupported type for AnchorIdlType, found: {:?}", self)
        }
    }
}

fn single_account_set_to_anchor(
    single_set: &IdlSingleAccountSet,
    name: &str,
    description: &ItemDescription,
) -> anchor::IdlInstructionAccount {
    anchor::IdlInstructionAccount {
        name: name.to_string(),
        docs: description.clone(),
        writable: single_set.writable,
        signer: single_set.signer,
        optional: single_set.optional,
        address: single_set.address.as_ref().map(ToString::to_string),
        pda: single_set.seeds.clone().map(Into::into),
        relations: vec![],
    }
}

impl From<IdlFindSeeds> for anchor::IdlPda {
    fn from(value: IdlFindSeeds) -> Self {
        anchor::IdlPda {
            program: value.program.as_ref().map(|p| {
                anchor::IdlSeed::Const(anchor::IdlSeedConst {
                    value: p.to_bytes().to_vec(),
                })
            }),
            seeds: value.seeds.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<IdlFindSeed> for anchor::IdlSeed {
    fn from(value: IdlFindSeed) -> Self {
        match value {
            IdlFindSeed::Const(value) => anchor::IdlSeed::Const(anchor::IdlSeedConst { value }),
            IdlFindSeed::AccountPath(path) => anchor::IdlSeed::Account(anchor::IdlSeedAccount {
                path,
                account: None,
            }),
        }
    }
}

fn convert_fields_to_anchor(
    fields: &[IdlStructField],
    idl_definition: &IdlDefinition,
) -> Result<Option<anchor::IdlDefinedFields>> {
    let Some(first_field) = fields.first() else {
        return Ok(None);
    };
    let is_named = first_field.path.is_some();
    let type_defs = fields
        .iter()
        .map(|f| {
            f.type_def
                .try_to_anchor(idl_definition)
                .context("Invalid field type")
        })
        .collect::<Result<Vec<_>>>()?;
    let defined_fields: anchor::IdlDefinedFields = if is_named {
        let named_fields = fields
            .iter()
            .zip(type_defs)
            .map(|(f, ty)| {
                Ok(anchor::IdlField {
                    name: f
                        .path
                        .as_ref()
                        .with_context(|| {
                            format!("Missing field name on named struct: {fields:#?}")
                        })?
                        .into(),
                    docs: f.description.clone(),
                    ty,
                })
            })
            .collect::<Result<Vec<anchor::IdlField>>>()?;
        anchor::IdlDefinedFields::Named(named_fields)
    } else {
        anchor::IdlDefinedFields::Tuple(type_defs)
    };
    Ok(Some(defined_fields))
}

fn convert_variants_to_anchor(
    variants: &[IdlEnumVariant],
    idl_definition: &IdlDefinition,
) -> Result<Vec<anchor::IdlEnumVariant>> {
    for (expected_discriminant, variant) in variants.iter().enumerate() {
        if variant.discriminant.len() != 1 {
            bail!("Enum discriminants must be u8")
        }
        let discriminant = variant.discriminant[0];
        if discriminant != expected_discriminant.try_into()? {
            bail!("Enum discriminants must be sequential for anchor compatibility")
        }
    }
    let variants: Vec<anchor::IdlEnumVariant> = variants
        .iter()
        .map(|variant| {
            let fields = variant
                .type_def
                .as_ref()
                .map::<Result<_>, _>(|ty| {
                    let res = if let IdlTypeDef::Struct(fields) = ty {
                        convert_fields_to_anchor(fields, idl_definition)?
                    } else {
                        let ty = ty.try_to_anchor(idl_definition)?;
                        Some(anchor::IdlDefinedFields::Tuple(vec![ty]))
                    };
                    Ok(res)
                })
                .transpose()?
                .flatten();

            Ok(anchor::IdlEnumVariant {
                name: variant.name.clone(),
                fields,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(variants)
}
