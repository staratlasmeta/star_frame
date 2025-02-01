use crate::account::IdlAccount;
use crate::seeds::IdlSeed;
use crate::ty::{IdlEnumVariant, IdlTypeDef};
use crate::{IdlDefinition, IdlDiscriminant, ItemInfo};
use anyhow::{bail, Result};
use codama_nodes::{
    AccountNode, ArrayTypeNode, BooleanTypeNode, BytesTypeNode, BytesValueNode, CamelCaseString,
    ConstantPdaSeedNode, DefinedTypeLinkNode, DefinedTypeNode, Docs, EnumEmptyVariantTypeNode,
    EnumTupleVariantTypeNode, EnumTypeNode, EnumVariantTypeNode, NumberFormat, NumberTypeNode,
    OptionTypeNode, PdaLinkNode, PdaNode, PdaSeedNode, ProgramNode, PublicKeyTypeNode, RootNode,
    SizePrefixTypeNode, StringTypeNode, StructFieldTypeNode, StructTypeNode, TupleTypeNode,
    TypeNode, TypeNodeTrait, ValueNode, VariablePdaSeedNode,
};

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
            bail!("Expected number type node, found {:?}", self)
        };
        Ok(number.clone())
    }
}

impl TryFrom<IdlDefinition> for RootNode {
    type Error = anyhow::Error;

    fn try_from(value: IdlDefinition) -> Result<Self, Self::Error> {
        let program_node = star_frame_to_codama(&value)?;
        Ok(RootNode::new(program_node))
    }
}

fn star_frame_to_codama(def: &IdlDefinition) -> Result<ProgramNode> {
    let mut program_node = ProgramNode {
        name: def.metadata.crate_metadata.name.as_str().into(),
        public_key: def.address.to_string(),
        version: def.metadata.crate_metadata.version.to_string(),
        origin: None,
        docs: def.metadata.crate_metadata.docs.clone().into(),
        defined_types: vec![], // done
        accounts: vec![],      // done
        pdas: vec![],          // done, todo: add "ghost-pda" support to star frame IDL definition
        instructions: vec![],  // todo
        errors: vec![],
    };

    let ctx = &mut Context;

    for (_, idl_type) in def.types.iter().chain(def.external_types.iter()) {
        program_node.defined_types.push(DefinedTypeNode {
            name: idl_type.info.codama_name(),
            docs: idl_type.info.codama_docs(),
            r#type: idl_type.type_def.try_to_codama(def, ctx)?,
        })
    }

    def.accounts.values().try_for_each(|account| {
        let account_node = convert_idl_account(account, def, &mut program_node, ctx)?;
        program_node.accounts.push(account_node);
        anyhow::Ok(())
    })?;
    Ok(program_node)
}

// todo: potentially add some error handling "context" info that gets passed around
pub struct Context;

trait TryToCodama<Output> {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        context: &mut Context,
    ) -> Result<Output>;
}

fn convert_idl_account(
    idl_account: &IdlAccount,
    idl_definition: &IdlDefinition,
    program_node: &mut ProgramNode,
    context: &mut Context,
) -> Result<AccountNode> {
    let defined_account = idl_account.type_id.get_defined(idl_definition)?;
    let info = &defined_account.info;
    let pda_link = if let Some(seeds) = &idl_account.seeds {
        let pda_node = PdaNode {
            name: info.codama_name(),
            docs: info.codama_docs(),
            program_id: None,
            seeds: seeds
                .iter()
                .map(|seed| {
                    let res = match seed {
                        IdlSeed::Const(bytes) => PdaSeedNode::Constant(ConstantPdaSeedNode {
                            r#type: TypeNode::Bytes(BytesTypeNode {}),
                            value: ValueNode::Bytes(BytesValueNode::base16(hex::encode(bytes))),
                        }),
                        IdlSeed::Variable {
                            name,
                            description,
                            ty,
                        } => PdaSeedNode::Variable(VariablePdaSeedNode {
                            name: name.as_str().into(),
                            docs: description.clone().into(),
                            r#type: ty.try_to_codama(idl_definition, context)?,
                        }),
                    };
                    Ok(res)
                })
                .collect::<Result<_>>()?,
        };
        program_node.pdas.push(pda_node);
        Some(PdaLinkNode {
            name: info.codama_name(),
            program: None,
        })
    } else {
        None
    };
    Ok(AccountNode {
        name: info.codama_name(),
        size: None,
        docs: info.codama_docs(),
        data: StructTypeNode::new(vec![StructFieldTypeNode::new(
            "", // todo: make this link node potentially or only do link if type_def isnt struct
            defined_account
                .type_def
                .try_to_codama(idl_definition, context)?,
        )])
        .into(),
        pda: pda_link,
        discriminators: vec![],
    })
}

impl TryToCodama<TypeNode> for IdlTypeDef {
    fn try_to_codama(&self, idl_def: &IdlDefinition, _context: &mut Context) -> Result<TypeNode> {
        use NumberFormat as Num;
        fn number(format: Num) -> TypeNode {
            NumberTypeNode::le(format).into_type_node()
        }
        let node = match self {
            IdlTypeDef::Defined(ty) => TypeNode::Link(DefinedTypeLinkNode {
                name: ty.get_defined(idl_def)?.info.name.as_str().into(),
                program: None,
            }),
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
            )
            .into_type_node(),
            IdlTypeDef::Pubkey => PublicKeyTypeNode {}.into_type_node(),
            IdlTypeDef::FixedPoint { ty, .. } => ty.try_to_codama(idl_def, _context)?,
            IdlTypeDef::Option(e) => {
                OptionTypeNode::new(e.try_to_codama(idl_def, _context)?).into_type_node()
            }
            IdlTypeDef::List { len_ty, item_ty } => {
                let len_ty = len_ty.try_to_codama(idl_def, _context)?.as_number()?;
                ArrayTypeNode::prefixed(item_ty.try_to_codama(idl_def, _context)?, len_ty)
                    .into_type_node()
            }
            IdlTypeDef::Array(ty, length) => {
                ArrayTypeNode::fixed(ty.try_to_codama(idl_def, _context)?, *length).into_type_node()
            }

            IdlTypeDef::Struct(fields) => StructTypeNode::new(
                fields
                    .iter()
                    .enumerate()
                    .map(|(index, f)| {
                        Ok(StructFieldTypeNode {
                            name: f.path.clone().unwrap_or(index.to_string()).into(),
                            default_value_strategy: None,
                            docs: f.description.clone().into(),
                            r#type: f.type_def.try_to_codama(idl_def, _context)?,
                            default_value: None,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?,
            )
            .into_type_node(),
            IdlTypeDef::Enum { variants, size } => EnumTypeNode {
                variants: variants
                    .iter()
                    .map(|variant| variant.try_to_codama(idl_def, _context))
                    .collect::<Result<_>>()?,
                size: size.try_to_codama(idl_def, _context)?.as_number()?.into(),
            }
            .into_type_node(),
            IdlTypeDef::Generic(_) => bail!("Generic types are not supported in Codama"),
        };
        Ok(node)
    }
}

impl TryToCodama<EnumVariantTypeNode> for IdlEnumVariant {
    fn try_to_codama(
        &self,
        idl_definition: &IdlDefinition,
        _context: &mut Context,
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
                            .collect::<Result<_>>()?,
                    )
                    .into(),
                })
            }
            Some(def) => {
                bail!("Idl type def not yet supported for enum variants: {def:?}")
            }
        };
        Ok(variant)
    }
}

fn discriminant_to_usize(discriminant: &IdlDiscriminant) -> Result<usize> {
    if discriminant.len() * 8 > std::mem::size_of::<usize>() {
        bail!(
            "Discriminant is too large. Max len: {}",
            std::mem::size_of::<usize>()
        )
    }
    let mut bytes = [0; std::mem::size_of::<usize>()];
    bytes[0..discriminant.len()][..].copy_from_slice(discriminant);
    Ok(usize::from_le_bytes(bytes))
}
