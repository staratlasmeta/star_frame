use crate::{
    account::IdlAccount,
    account_set::{IdlAccountSetDef, IdlAccountSetStructField, IdlSingleAccountSet},
    instruction::IdlInstruction,
    seeds::{IdlFindSeed, IdlFindSeeds, IdlSeed},
    ty::{IdlEnumVariant, IdlTypeDef},
    IdlDefinition, IdlDiscriminant, ItemDescription, ItemInfo, Result,
};
use codama_nodes::{
    AccountNode, AccountValueNode, ArgumentValueNode, ArrayTypeNode, BooleanTypeNode,
    BytesTypeNode, BytesValueNode, CamelCaseString, ConstantPdaSeedNode, DefaultValueStrategy,
    DefinedTypeLinkNode, DefinedTypeNode, DiscriminatorNode, Docs, EnumEmptyVariantTypeNode,
    EnumTupleVariantTypeNode, EnumTypeNode, EnumVariantTypeNode, FieldDiscriminatorNode,
    FixedSizeTypeNode, InstructionAccountNode, InstructionNode, InstructionRemainingAccountsNode,
    InstructionRemainingAccountsNodeValue, MapTypeNode, NumberFormat, NumberTypeNode,
    OptionTypeNode, PdaLinkNode, PdaNode, PdaSeedNode, PdaSeedValueNode, PdaValueNode,
    ProgramLinkNode, PublicKeyTypeNode, PublicKeyValueNode, SetTypeNode, SizePrefixTypeNode,
    StringTypeNode, StructFieldTypeNode, StructTypeNode, TupleTypeNode, TypeNode, TypeNodeTrait,
    VariablePdaSeedNode,
};
pub use codama_nodes::{ErrorNode, NodeTrait, ProgramNode};
use itertools::Itertools;

impl ItemInfo {
    fn codama_name(&self) -> CamelCaseString {
        self.name.as_str().into()
    }

    fn codama_docs(&self) -> Docs {
        self.description.clone().into()
    }
}

trait CodamaNodeExt {
    fn as_number(&self) -> Result<NumberTypeNode>;
}

impl CodamaNodeExt for TypeNode {
    fn as_number(&self) -> Result<NumberTypeNode> {
        let TypeNode::Number(number) = self else {
            return Err(crate::Error::ExpectedNumberTypeNode(format!("{self:?}")));
        };
        Ok(number.clone())
    }
}

const DISCRIMINATOR_NAME: &str = "discriminator";

fn discriminator_info(discriminant: &IdlDiscriminant) -> (StructFieldTypeNode, DiscriminatorNode) {
    (
        StructFieldTypeNode {
            name: DISCRIMINATOR_NAME.into(),
            default_value_strategy: Some(DefaultValueStrategy::Omitted),
            docs: Default::default(),
            r#type: FixedSizeTypeNode::new(BytesTypeNode {}, discriminant.len()).into(),
            default_value: Some(BytesValueNode::base16(hex::encode(discriminant)).into()),
        },
        DiscriminatorNode::Field(FieldDiscriminatorNode {
            name: DISCRIMINATOR_NAME.into(),
            offset: 0,
        }),
    )
}

impl TryFrom<IdlDefinition> for ProgramNode {
    type Error = crate::Error;

    fn try_from(def: IdlDefinition) -> Result<Self> {
        let ctx = &mut TryToCodamaContext;

        let (accounts, maybe_pdas): (_, Vec<_>) = def
            .accounts
            .values()
            .map(|account| {
                account.try_to_codama(&def, ctx).map_err(|e| {
                    crate::Error::CodamaConversion(format!("Failed to convert account: {e}"))
                })
            })
            .try_collect()?;

        let pdas = maybe_pdas.into_iter().flatten().collect();

        let mut defined_types: Vec<_> = def
            .types
            .iter()
            .filter(|(source, _)| {
                !def.accounts.contains_key(*source) && !def.instructions.contains_key(*source)
            })
            .map(|(_source, idl_type)| {
                Ok(DefinedTypeNode {
                    name: idl_type.info.codama_name(),
                    docs: idl_type.info.codama_docs(),
                    r#type: idl_type.type_def.try_to_codama(&def, ctx)?,
                })
            })
            .try_collect()?;
        defined_types.sort_by_key(|ty| ty.name.to_string());

        let mut instructions: Vec<_> = def
            .instructions
            .values()
            .map(|instruction| instruction.try_to_codama(&def, ctx))
            .try_collect()?;
        instructions.sort_by_key(|ix: &InstructionNode| ix.name.to_string());

        Ok(ProgramNode {
            name: def.metadata.crate_metadata.name.as_str().into(),
            public_key: def.address.to_string(),
            version: def.metadata.crate_metadata.version.to_string(),
            origin: None,
            docs: def.metadata.crate_metadata.docs.clone().into(),
            defined_types,
            accounts,
            pdas, // todo: add "ghost-pda" support to star frame IDL definition
            instructions,
            errors: def.errors,
        })
    }
}

// todo: potentially add some error handling "context" info that gets passed around
pub struct TryToCodamaContext;

trait TryToCodama<Output> {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        context: &mut TryToCodamaContext,
    ) -> Result<Output>;
}

fn ensure_struct_node(type_node: TypeNode) -> Result<StructTypeNode> {
    match type_node {
        TypeNode::Struct(struct_node) => Ok(struct_node),
        TypeNode::Tuple(tuple_node) if tuple_node.items.is_empty() => {
            Ok(StructTypeNode { fields: vec![] })
        }
        _ => Err(crate::Error::UnsupportedAccountType(format!(
            "{:?}",
            type_node
        ))),
    }
}

impl TryToCodama<InstructionNode> for IdlInstruction {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        context: &mut TryToCodamaContext,
    ) -> Result<InstructionNode> {
        let idl_type = self.definition.type_id.get_defined(idl_definition)?;
        let struct_node =
            ensure_struct_node(idl_type.type_def.try_to_codama(idl_definition, context)?)?;

        let (discriminator_field, discriminator_node) = discriminator_info(&self.discriminant);
        let mut arguments = vec![discriminator_field.into()];
        arguments.extend(struct_node.fields.into_iter().map(Into::into));
        let (accounts, remaining_accounts) = self
            .definition
            .account_set
            .try_to_codama(idl_definition, context)?;

        let instruction_node = InstructionNode {
            name: idl_type.info.codama_name(),
            docs: idl_type.info.codama_docs(),
            accounts,
            remaining_accounts,
            arguments,
            discriminators: vec![discriminator_node],
            ..Default::default()
        };
        Ok(instruction_node)
    }
}

type CodamaInstructionInfo = (
    Vec<InstructionAccountNode>,
    Vec<InstructionRemainingAccountsNode>,
);

#[derive(Debug, Clone, Default)]
struct PathInfo {
    paths: Vec<String>,
}

impl PathInfo {
    fn name(&self) -> CamelCaseString {
        self.paths.join(" ").into()
    }
    fn create_next(&self, name: Option<&str>, index: usize) -> Self {
        let mut paths = self.clone();
        let name = name.map(ToString::to_string);
        paths.paths.push(name.unwrap_or_else(|| index.to_string()));
        paths
    }
}

fn seeds_to_pda_value_node(seeds: &IdlFindSeeds, paths: &PathInfo) -> PdaValueNode {
    let mut paths = paths.clone();
    let name = paths.name();
    paths.paths.pop();
    let (pda_node_seeds, lookup_seeds): (_, Vec<_>) = seeds
        .seeds
        .iter()
        .enumerate()
        .map(|(index, seed)| match seed {
            IdlFindSeed::Const(bytes) => {
                let value: PdaSeedNode = ConstantPdaSeedNode::new(
                    BytesTypeNode {},
                    BytesValueNode::base16(hex::encode(bytes)),
                )
                .into();
                (value, None)
            }
            IdlFindSeed::AccountPath(account_path) => {
                let name = format!("{account_path}{index}");
                let value = VariablePdaSeedNode::new(name.clone(), PublicKeyTypeNode {}).into();
                // Account paths that start with a colon are interpreted as root paths
                let path_name = if let Some(stripped) = account_path.strip_prefix(':') {
                    stripped.into()
                } else {
                    paths.create_next(Some(account_path), index).name()
                };
                let lookup = PdaSeedValueNode {
                    name: name.into(),
                    value: AccountValueNode { name: path_name }.into(),
                };
                (value, Some(lookup))
            }
        })
        .collect();

    PdaValueNode {
        pda: PdaNode {
            name,
            docs: Default::default(),
            program_id: seeds.program.as_ref().map(ToString::to_string),
            seeds: pda_node_seeds,
        }
        .into(),
        seeds: lookup_seeds.into_iter().flatten().collect(),
    }
}

fn single_set_to_account_node(
    single_set: &IdlSingleAccountSet,
    paths: &PathInfo,
    description: &ItemDescription,
) -> InstructionAccountNode {
    let default_value = match (single_set.address, &single_set.seeds) {
        (Some(address), _) => Some(PublicKeyValueNode::new(address.to_string()).into()),
        (None, Some(seeds)) => Some(seeds_to_pda_value_node(seeds, paths).into()),
        _ => None,
    };
    InstructionAccountNode {
        name: paths.name(),
        is_writable: single_set.writable,
        is_signer: single_set.signer.into(),
        is_optional: single_set.optional,
        docs: (*description).clone().into(),
        default_value,
    }
}

fn instruction_account_to_remaining(
    account: InstructionAccountNode,
) -> Result<InstructionRemainingAccountsNode> {
    if account.default_value.is_some() {
        return Err(crate::Error::RemainingAccountsCannotHaveDefaults(format!(
            "{:?}",
            account
        )));
    }
    Ok(InstructionRemainingAccountsNode {
        is_optional: account.is_optional,
        is_signer: account.is_signer,
        is_writable: account.is_writable,
        docs: account.docs,
        value: InstructionRemainingAccountsNodeValue::Argument(ArgumentValueNode {
            name: account.name,
        }),
    })
}

impl TryToCodama<CodamaInstructionInfo> for (&IdlAccountSetStructField, &PathInfo) {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        context: &mut TryToCodamaContext,
    ) -> Result<CodamaInstructionInfo> {
        let (field, paths) = self;
        let account_set_def = &field.account_set_def;
        let def = match account_set_def {
            IdlAccountSetDef::Defined(_) => {
                let set = account_set_def.get_defined(idl_definition)?;
                (&set.account_set_def, *paths).try_to_codama(idl_definition, context)?
            }
            IdlAccountSetDef::Single(single_set) => {
                let single = single_set_to_account_node(single_set, paths, &field.description);
                (vec![single], vec![])
            }
            IdlAccountSetDef::Many { account_set, .. } => {
                let mut set: IdlAccountSetDef = account_set.as_ref().clone();
                let single = set
                    .single()
                    .map_err(|_| crate::Error::ManySetsMustBeSingle)?;
                let single = single_set_to_account_node(single, paths, &field.description);
                let remaining = instruction_account_to_remaining(single)?;
                (vec![], vec![remaining])
            }
            IdlAccountSetDef::Struct(_) | IdlAccountSetDef::Or(_) => {
                return (account_set_def, *paths).try_to_codama(idl_definition, context)
            }
        };
        Ok(def)
    }
}

impl TryToCodama<CodamaInstructionInfo> for (&IdlAccountSetDef, &PathInfo) {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        _context: &mut TryToCodamaContext,
    ) -> Result<CodamaInstructionInfo> {
        let (account_set_def, paths) = self;
        let fields = match account_set_def {
            IdlAccountSetDef::Defined(_) => {
                let set = account_set_def.get_defined(idl_definition)?;
                (&set.account_set_def, *paths).try_to_codama(idl_definition, _context)?
            }
            IdlAccountSetDef::Struct(struct_fields) => {
                let mut fields = vec![];
                let mut remaining = vec![];
                for (index, field) in struct_fields.iter().enumerate() {
                    let new_path = paths.create_next(field.path.as_deref(), index);
                    let (new_fields, new_remaining) =
                        (field, &new_path).try_to_codama(idl_definition, _context)?;
                    if !remaining.is_empty() && !new_fields.is_empty() {
                        return Err(crate::Error::ManyAccountSetsMustComeLast);
                    }
                    fields.extend(new_fields);
                    remaining.extend(new_remaining);
                }
                (fields, remaining)
            }
            _ => {
                return Err(crate::Error::UnsupportedAccountSetType(format!(
                    "{:?}",
                    self.0
                )))
            }
        };
        Ok(fields)
    }
}

impl TryToCodama<CodamaInstructionInfo> for IdlAccountSetDef {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        context: &mut TryToCodamaContext,
    ) -> Result<CodamaInstructionInfo> {
        (self, &PathInfo::default()).try_to_codama(idl_definition, context)
    }
}

impl TryToCodama<PdaSeedNode> for IdlSeed {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        context: &mut TryToCodamaContext,
    ) -> Result<PdaSeedNode> {
        let res = match self {
            IdlSeed::Const(bytes) => ConstantPdaSeedNode::new(
                BytesTypeNode {},
                BytesValueNode::base16(hex::encode(bytes)),
            )
            .into(),
            IdlSeed::Variable {
                name,
                description,
                ty,
            } => VariablePdaSeedNode {
                name: name.as_str().into(),
                docs: description.clone().into(),
                r#type: ty.try_to_codama(idl_definition, context)?,
            }
            .into(),
        };
        Ok(res)
    }
}

impl TryToCodama<(AccountNode, Option<PdaNode>)> for IdlAccount {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        context: &mut TryToCodamaContext,
    ) -> Result<(AccountNode, Option<PdaNode>)> {
        let defined_account = self.type_id.get_defined(idl_definition)?;

        let info = &defined_account.info;

        let mut struct_node = ensure_struct_node(
            defined_account
                .type_def
                .try_to_codama(idl_definition, context)?,
        )?;

        let pda_node = self
            .seeds
            .as_ref()
            .map(|seeds| {
                Ok(PdaNode {
                    name: info.codama_name(),
                    docs: info.codama_docs(),
                    program_id: None,
                    seeds: seeds
                        .iter()
                        .map(|seed| {
                            seed.try_to_codama(idl_definition, context).map_err(|e| {
                                crate::Error::CodamaConversion(format!(
                                    "Failed to convert seed: {e}"
                                ))
                            })
                        })
                        .try_collect()?,
                })
            })
            .transpose()?;

        let (discriminator_field, discriminator_node) = discriminator_info(&self.discriminant);
        struct_node.fields.insert(0, discriminator_field);

        Ok((
            AccountNode {
                name: info.codama_name(),
                size: None,
                docs: info.codama_docs(),
                data: struct_node.into(),
                pda: pda_node.is_some().then(|| PdaLinkNode {
                    name: info.codama_name(),
                    program: None,
                }),
                discriminators: vec![discriminator_node],
            },
            pda_node,
        ))
    }
}

impl TryToCodama<TypeNode> for IdlTypeDef {
    fn try_to_codama(
        &self,
        idl_def: &IdlDefinition,
        _context: &mut TryToCodamaContext,
    ) -> Result<TypeNode> {
        use NumberFormat as Num;
        fn number(format: Num) -> TypeNode {
            NumberTypeNode::le(format).into_type_node()
        }
        let node = match self {
            IdlTypeDef::Defined(ty) => {
                // todo: Right now if we're linking external types this may break if the external type is actually an account.
                //  Codama needs to have accounts be DefinedTypeLinkNodes first or a discriminator will magically appear in any
                //  of these types
                let defined = ty.get_defined(idl_def).map_err(|e| crate::Error::CodamaConversion(format!(
                    "Failed to get defined type: {e}"
                )))?;
                let name = defined.info.codama_name();
                let program = ty.namespace.as_ref().map(|namespace| ProgramLinkNode {
                    name: namespace.to_string().into(),
                });
                TypeNode::Link(DefinedTypeLinkNode { name, program })
            }
            IdlTypeDef::Bool => BooleanTypeNode::default().into_type_node(),
            IdlTypeDef::U8 => number(Num::U8),
            IdlTypeDef::I8 => number(Num::I8),
            IdlTypeDef::U16 => number(Num::U16),
            IdlTypeDef::I16 => number(Num::I16),
            IdlTypeDef::U32 => number(Num::U32),
            IdlTypeDef::I32 => number(Num::I32),
            IdlTypeDef::F32 => number(Num::F32),
            IdlTypeDef::U64 => number(Num::U64),
            IdlTypeDef::I64 => number(Num::I64),
            IdlTypeDef::F64 => number(Num::F64),
            IdlTypeDef::U128 => number(Num::U128),
            IdlTypeDef::I128 => number(Num::I128),
            IdlTypeDef::String => SizePrefixTypeNode::<TypeNode>::new(
                StringTypeNode::utf8(),
                NumberTypeNode::le(Num::U32),
            ).into_type_node(),
            IdlTypeDef::Pubkey => PublicKeyTypeNode {}.into_type_node(),
            IdlTypeDef::FixedPoint { ty, .. } => ty.try_to_codama(idl_def, _context)?,
            IdlTypeDef::Option { ty, fixed } =>
                OptionTypeNode {
                    fixed: *fixed,
                    item: Box::new(ty.try_to_codama(idl_def, _context)?),
                    prefix: NumberTypeNode::le(Num::U8).into(),
                }.into_type_node(),
            IdlTypeDef::List { len_ty, item_ty } => {
                let len_ty = len_ty.try_to_codama(idl_def, _context)?.as_number()?;
                ArrayTypeNode::prefixed(item_ty.try_to_codama(idl_def, _context)?, len_ty)
                    .into_type_node()
            }
            IdlTypeDef::Array(ty, length) => {
                ArrayTypeNode::fixed(ty.try_to_codama(idl_def, _context)?, *length).into_type_node()
            }

            IdlTypeDef::Struct(fields) => {
                let named = fields.first().is_some_and(|f| f.path.is_some());
                if named {
                    StructTypeNode::new(
                        fields
                            .iter()
                            .map(|f|{
                                Ok(StructFieldTypeNode {
                                    name: f.path.clone().ok_or(crate::Error::MissingNameOnNamedField)?.into(),
                                    default_value_strategy: None,
                                    docs: f.description.clone().into(),
                                    r#type: f.type_def.try_to_codama(idl_def, _context)?,
                                    default_value: None,
                                })
                            })
                            .try_collect()?,
                    ).into_type_node()
                } else {
                    TupleTypeNode::new(
                        fields
                            .iter()
                            .map(|f| f.type_def.try_to_codama(idl_def, _context))
                            .try_collect()?,
                    ).into_type_node()
                }
            }
            IdlTypeDef::Enum { variants, size } => EnumTypeNode {
                variants: variants
                    .iter()
                    .map(|variant| variant.try_to_codama(idl_def, _context))
                    .try_collect()?,
                size: size.try_to_codama(idl_def, _context)?.as_number()?.into(),
            }.into_type_node(),
            IdlTypeDef::Set { len_ty, item_ty } => {
                let len_ty = len_ty.try_to_codama(idl_def, _context)?.as_number()?;
                SetTypeNode::prefixed(item_ty.try_to_codama(idl_def, _context)?, len_ty)
                    .into_type_node()
            }
            IdlTypeDef::Map {
                len_ty,
                key_ty,
                value_ty,
            } => {
                let len_ty = len_ty.try_to_codama(idl_def, _context)?.as_number()?;
                MapTypeNode::prefixed(
                    key_ty.try_to_codama(idl_def, _context)?,
                    value_ty.try_to_codama(idl_def, _context)?,
                    len_ty,
                ).into_type_node()
            }
            IdlTypeDef::RemainingBytes => {
                ArrayTypeNode::remainder(number(Num::U8)).into_type_node()
            }
            IdlTypeDef::UnsizedList {
                len_ty,
                offset_ty,
                item_ty,
            } => StructTypeNode::new(vec![
                StructFieldTypeNode {
                    name: "unsized_len".into(),
                    default_value_strategy: Some(DefaultValueStrategy::Omitted),
                    docs: vec!["The total size of the unsized bytes in the list".into()].into(),
                    r#type: len_ty.try_to_codama(idl_def, _context)?,
                    default_value: None,
                },
                StructFieldTypeNode {
                    name: "offset_list".into(),
                    default_value_strategy: Some(DefaultValueStrategy::Omitted),
                    docs: vec!["The list of items containing offsets, and potentially extra metadata, for the start of each element in the unsized list".into()].into(),
                    r#type: IdlTypeDef::List { item_ty: offset_ty.clone(), len_ty: len_ty.clone() }.try_to_codama(idl_def, _context)?,
                    default_value: None,
                },
                StructFieldTypeNode {
                    name: "unsized_list".into(),
                    default_value_strategy: Some(DefaultValueStrategy::Omitted),
                    docs: vec!["The list of byte offsets for the start of each element in the unsized list".into()].into(),
                    r#type: IdlTypeDef::List { item_ty: item_ty.clone(), len_ty: len_ty.clone() }.try_to_codama(idl_def, _context)?,
                    default_value: None,
                },
            ]).into_type_node(),
            IdlTypeDef::Generic(_) => return Err(crate::Error::GenericTypesNotSupported),
        };
        Ok(node)
    }
}

impl TryToCodama<EnumVariantTypeNode> for IdlEnumVariant {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        _context: &mut TryToCodamaContext,
    ) -> Result<EnumVariantTypeNode> {
        let discriminator = Some(discriminant_to_usize(&self.discriminant)?);
        let name = self.name.as_str().into();
        let variant = match &self.type_def {
            None => EnumVariantTypeNode::Empty(EnumEmptyVariantTypeNode {
                name,
                discriminator,
            }),
            Some(IdlTypeDef::Struct(fields)) => {
                // TODO: potentially handle enums variants with named fields better
                EnumVariantTypeNode::Tuple(EnumTupleVariantTypeNode {
                    name,
                    discriminator,
                    tuple: TupleTypeNode::new(
                        fields
                            .iter()
                            .map(|f| f.type_def.try_to_codama(idl_definition, _context))
                            .try_collect()?,
                    )
                    .into(),
                })
            }
            Some(def) => {
                return Err(crate::Error::UnsupportedEnumVariantType(format!("{def:?}")));
            }
        };
        Ok(variant)
    }
}

fn discriminant_to_usize(discriminant: &IdlDiscriminant) -> Result<usize> {
    if discriminant.len() * 8 > std::mem::size_of::<usize>() {
        return Err(crate::Error::DiscriminantTooLarge(
            std::mem::size_of::<usize>(),
        ));
    }
    let mut bytes = [0; std::mem::size_of::<usize>()];
    bytes[0..discriminant.len()][..].copy_from_slice(discriminant);
    Ok(usize::from_le_bytes(bytes))
}
