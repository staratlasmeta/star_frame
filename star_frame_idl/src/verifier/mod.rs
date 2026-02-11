//! Structural verification for one or more [`IdlDefinition`] values.
//!
//! This verifier is intentionally structural-only:
//! - It validates namespace hygiene, reference resolution, shape constraints, and generic arity.
//! - It does not attempt Codama semantic validation or audit-grade program logic validation.
//!
//! Verification is fail-closed: invalid graphs return `Err` with stable rule identifiers
//! (`SFIDL001`-`SFIDL011`) embedded in the diagnostic message.

use std::collections::BTreeMap;

use crate::{
    account::IdlAccountId,
    account_set::{IdlAccountSetDef, IdlAccountSetId, IdlSingleAccountSet},
    seeds::IdlSeed,
    ty::{IdlType, IdlTypeDef, IdlTypeId},
    Error, IdlDefinition, Result,
};

const RULE_EMPTY_NAMESPACE: &str = "SFIDL001";
const RULE_DUPLICATE_NAMESPACE: &str = "SFIDL002";
const RULE_MISSING_NAMESPACE: &str = "SFIDL003";
const RULE_MISSING_TYPE: &str = "SFIDL004";
const RULE_TYPE_GENERIC_ARITY: &str = "SFIDL005";
const RULE_MISSING_ACCOUNT_SET: &str = "SFIDL006";
const RULE_ACCOUNT_SET_TYPE_ARITY: &str = "SFIDL007";
const RULE_ACCOUNT_SET_ACCOUNT_ARITY: &str = "SFIDL008";
const RULE_MISSING_ACCOUNT: &str = "SFIDL009";
const RULE_MANY_BOUNDS: &str = "SFIDL010";
const RULE_EMPTY_OR: &str = "SFIDL011";

/// Controls how namespaced references are resolved during structural verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMode {
    /// Verifies structural correctness while allowing self-contained IDL definitions
    /// to satisfy namespaced type/account references from local embedded tables.
    Compatibility,
    /// Verifies full definition graphs where namespaced references must resolve
    /// to a namespace present in the provided definition set.
    StrictGraph,
}

#[derive(Debug)]
struct NamespaceIndex<'a> {
    definitions: BTreeMap<&'a str, &'a IdlDefinition>,
}

impl<'a> NamespaceIndex<'a> {
    fn build(definitions: &[&'a IdlDefinition]) -> Result<Self> {
        let mut by_namespace = BTreeMap::new();
        for definition in definitions {
            let namespace = definition.metadata.crate_metadata.name.trim();
            if namespace.is_empty() {
                return Err(verifier_err(
                    RULE_EMPTY_NAMESPACE,
                    "IDL definition namespace cannot be empty",
                ));
            }
            if by_namespace.insert(namespace, *definition).is_some() {
                return Err(verifier_err(
                    RULE_DUPLICATE_NAMESPACE,
                    format!("Duplicate IDL definition namespace `{namespace}`"),
                ));
            }
        }
        Ok(Self {
            definitions: by_namespace,
        })
    }

    fn by_namespace(&self, namespace: &str) -> Option<&'a IdlDefinition> {
        self.definitions.get(namespace).copied()
    }

    fn require_namespace(&self, namespace: &str, context: &str) -> Result<&'a IdlDefinition> {
        self.by_namespace(namespace).ok_or_else(|| {
            verifier_err(
                RULE_MISSING_NAMESPACE,
                format!("{context}: namespace `{namespace}` was not provided"),
            )
        })
    }
}

fn verifier_err(rule: &'static str, message: impl Into<String>) -> Error {
    Error::Custom(format!("Verifier error [{rule}]: {}", message.into()))
}

/// Verifies structural correctness using [`VerificationMode::Compatibility`].
///
/// This is the recommended default for existing callers that rely on
/// self-contained definitions where namespaced references can be satisfied by
/// local `types`/`external_types`/`accounts`.
pub fn verify_idl_definitions<'a, I>(def_set: I) -> Result<()>
where
    I: IntoIterator,
    I::IntoIter: Iterator<Item = &'a IdlDefinition> + Clone,
{
    verify_idl_definitions_with_mode(def_set, VerificationMode::Compatibility)
}

/// Verifies a definition set in strict graph mode.
///
/// In `StrictGraph`, namespaced references must resolve through the provided
/// definition set. Embedded local copies alone do not satisfy missing namespaces.
pub fn verify_idl_definitions_strict<'a, I>(def_set: I) -> Result<()>
where
    I: IntoIterator,
    I::IntoIter: Iterator<Item = &'a IdlDefinition> + Clone,
{
    verify_idl_definitions_with_mode(def_set, VerificationMode::StrictGraph)
}

/// Verifies structural correctness for one or more IDL definitions.
///
/// Behavior is controlled by [`VerificationMode`]:
/// - [`VerificationMode::Compatibility`] (default behavior of
///   [`verify_idl_definitions`]) first resolves namespaced type/account
///   references against the current definition's local tables, then falls back
///   to provided namespace definitions.
/// - [`VerificationMode::StrictGraph`] requires namespaced references to resolve
///   via definitions provided in `def_set`.
///
/// Returns `Err` on the first structural rule violation, with a stable rule ID
/// (`SFIDL001`-`SFIDL011`) included in the error message.
pub fn verify_idl_definitions_with_mode<'a, I>(def_set: I, mode: VerificationMode) -> Result<()>
where
    I: IntoIterator,
    I::IntoIter: Iterator<Item = &'a IdlDefinition> + Clone,
{
    let definitions: Vec<&IdlDefinition> = def_set.into_iter().collect();
    let namespace_index = NamespaceIndex::build(&definitions)?;
    for definition in definitions {
        verify_definition(definition, &namespace_index, mode)?;
    }
    Ok(())
}

fn verify_definition<'a>(
    definition: &'a IdlDefinition,
    namespace_index: &NamespaceIndex<'a>,
    mode: VerificationMode,
) -> Result<()> {
    let namespace = definition.metadata.crate_metadata.name.as_str();

    for (source, idl_type) in definition
        .types
        .iter()
        .chain(definition.external_types.iter())
    {
        verify_type_def(
            &idl_type.type_def,
            definition,
            namespace_index,
            mode,
            &format!("namespace `{namespace}` type `{source}`"),
        )?;
    }

    for (source, account_set) in &definition.account_sets {
        verify_account_set_def(
            &account_set.account_set_def,
            definition,
            namespace_index,
            mode,
            &format!("namespace `{namespace}` account_set `{source}`"),
        )?;
    }

    for (source, account) in &definition.accounts {
        let context = format!("namespace `{namespace}` account `{source}`");
        verify_type_id(
            &account.type_id,
            definition,
            namespace_index,
            mode,
            &context,
        )?;
        if let Some(seeds) = &account.seeds {
            for (seed_index, seed) in seeds.0.iter().enumerate() {
                if let IdlSeed::Variable { ty, .. } = seed {
                    verify_type_def(
                        ty,
                        definition,
                        namespace_index,
                        mode,
                        &format!("{context} seed[{seed_index}]"),
                    )?;
                }
            }
        }
    }

    for (source, instruction) in &definition.instructions {
        let context = format!("namespace `{namespace}` instruction `{source}`");
        verify_type_id(
            &instruction.definition.type_id,
            definition,
            namespace_index,
            mode,
            &context,
        )?;
        verify_account_set_def(
            &instruction.definition.account_set,
            definition,
            namespace_index,
            mode,
            &format!("{context} account_set"),
        )?;
    }

    Ok(())
}

fn verify_type_id<'a>(
    type_id: &IdlTypeId,
    current: &'a IdlDefinition,
    namespace_index: &NamespaceIndex<'a>,
    mode: VerificationMode,
    context: &str,
) -> Result<&'a IdlType> {
    let resolved_type = match type_id.namespace.as_deref() {
        None => current.get_type(&type_id.source),
        Some(namespace) => match mode {
            VerificationMode::Compatibility => current.get_type(&type_id.source).or_else(|| {
                namespace_index
                    .by_namespace(namespace)
                    .and_then(|definition| definition.get_type(&type_id.source))
            }),
            VerificationMode::StrictGraph => namespace_index
                .require_namespace(namespace, context)?
                .get_type(&type_id.source),
        },
    }
    .ok_or_else(|| match type_id.namespace.as_deref() {
        Some(namespace)
            if matches!(mode, VerificationMode::Compatibility)
                && namespace_index.by_namespace(namespace).is_none()
                && current.get_type(&type_id.source).is_none() =>
        {
            verifier_err(
                RULE_MISSING_NAMESPACE,
                format!("{context}: namespace `{namespace}` was not provided"),
            )
        }
        Some(namespace) => verifier_err(
            RULE_MISSING_TYPE,
            format!(
                "{context}: type `{}` not found in namespace `{namespace}`",
                type_id.source
            ),
        ),
        None => verifier_err(
            RULE_MISSING_TYPE,
            format!(
                "{context}: type `{}` not found in namespace `{}`",
                type_id.source, current.metadata.crate_metadata.name
            ),
        ),
    })?;

    if type_id.provided_generics.len() != resolved_type.generics.len() {
        return Err(verifier_err(
            RULE_TYPE_GENERIC_ARITY,
            format!(
                "{context}: type `{}` expected {} generic args, found {}",
                type_id.source,
                resolved_type.generics.len(),
                type_id.provided_generics.len()
            ),
        ));
    }

    for (generic_index, provided_generic) in type_id.provided_generics.iter().enumerate() {
        verify_type_def(
            provided_generic,
            current,
            namespace_index,
            mode,
            &format!("{context} type_generic[{generic_index}]"),
        )?;
    }

    Ok(resolved_type)
}

fn verify_account_set_id<'a>(
    account_set_id: &IdlAccountSetId,
    current: &'a IdlDefinition,
    namespace_index: &NamespaceIndex<'a>,
    mode: VerificationMode,
    context: &str,
) -> Result<()> {
    let target_set = current
        .account_sets
        .get(&account_set_id.source)
        .ok_or_else(|| {
            verifier_err(
                RULE_MISSING_ACCOUNT_SET,
                format!(
                    "{context}: account set `{}` not found in namespace `{}`",
                    account_set_id.source, current.metadata.crate_metadata.name
                ),
            )
        })?;

    if account_set_id.provided_type_generics.len() != target_set.type_generics.len() {
        return Err(verifier_err(
            RULE_ACCOUNT_SET_TYPE_ARITY,
            format!(
                "{context}: account set `{}` expected {} type generics, found {}",
                account_set_id.source,
                target_set.type_generics.len(),
                account_set_id.provided_type_generics.len()
            ),
        ));
    }

    if account_set_id.provided_account_generics.len() != target_set.account_generics.len() {
        return Err(verifier_err(
            RULE_ACCOUNT_SET_ACCOUNT_ARITY,
            format!(
                "{context}: account set `{}` expected {} account generics, found {}",
                account_set_id.source,
                target_set.account_generics.len(),
                account_set_id.provided_account_generics.len()
            ),
        ));
    }

    for (generic_index, provided_generic) in
        account_set_id.provided_type_generics.iter().enumerate()
    {
        verify_type_def(
            provided_generic,
            current,
            namespace_index,
            mode,
            &format!("{context} account_set_type_generic[{generic_index}]"),
        )?;
    }

    for (generic_index, provided_generic) in
        account_set_id.provided_account_generics.iter().enumerate()
    {
        verify_account_set_def(
            provided_generic,
            current,
            namespace_index,
            mode,
            &format!("{context} account_set_account_generic[{generic_index}]"),
        )?;
    }

    Ok(())
}

fn verify_account_id<'a>(
    account_id: &IdlAccountId,
    current: &'a IdlDefinition,
    namespace_index: &NamespaceIndex<'a>,
    mode: VerificationMode,
    context: &str,
) -> Result<()> {
    let account_exists = match account_id.namespace.as_deref() {
        None => current.accounts.contains_key(&account_id.source),
        Some(namespace) => match mode {
            VerificationMode::Compatibility => {
                current.accounts.contains_key(&account_id.source)
                    || namespace_index
                        .by_namespace(namespace)
                        .is_some_and(|definition| {
                            definition.accounts.contains_key(&account_id.source)
                        })
            }
            VerificationMode::StrictGraph => namespace_index
                .require_namespace(namespace, context)?
                .accounts
                .contains_key(&account_id.source),
        },
    };

    if !account_exists {
        return Err(match account_id.namespace.as_deref() {
            Some(namespace)
                if matches!(mode, VerificationMode::Compatibility)
                    && !current.accounts.contains_key(&account_id.source)
                    && namespace_index.by_namespace(namespace).is_none() =>
            {
                verifier_err(
                    RULE_MISSING_NAMESPACE,
                    format!("{context}: namespace `{namespace}` was not provided"),
                )
            }
            Some(namespace) => verifier_err(
                RULE_MISSING_ACCOUNT,
                format!(
                    "{context}: account `{}` not found in namespace `{namespace}`",
                    account_id.source
                ),
            ),
            None => verifier_err(
                RULE_MISSING_ACCOUNT,
                format!(
                    "{context}: account `{}` not found in namespace `{}`",
                    account_id.source, current.metadata.crate_metadata.name
                ),
            ),
        });
    }

    Ok(())
}

fn verify_single_account_set<'a>(
    single_set: &IdlSingleAccountSet,
    current: &'a IdlDefinition,
    namespace_index: &NamespaceIndex<'a>,
    mode: VerificationMode,
    context: &str,
) -> Result<()> {
    for (account_index, account_id) in single_set.program_accounts.iter().enumerate() {
        verify_account_id(
            account_id,
            current,
            namespace_index,
            mode,
            &format!("{context} program_account[{account_index}]"),
        )?;
    }
    Ok(())
}

fn verify_account_set_def<'a>(
    account_set_def: &IdlAccountSetDef,
    current: &'a IdlDefinition,
    namespace_index: &NamespaceIndex<'a>,
    mode: VerificationMode,
    context: &str,
) -> Result<()> {
    match account_set_def {
        IdlAccountSetDef::Defined(id) => {
            verify_account_set_id(id, current, namespace_index, mode, context)
        }
        IdlAccountSetDef::Single(single_set) => {
            verify_single_account_set(single_set, current, namespace_index, mode, context)
        }
        IdlAccountSetDef::Struct(fields) => {
            for (field_index, field) in fields.iter().enumerate() {
                verify_account_set_def(
                    &field.account_set_def,
                    current,
                    namespace_index,
                    mode,
                    &format!("{context} field[{field_index}]"),
                )?;
            }
            Ok(())
        }
        IdlAccountSetDef::Many {
            account_set,
            min,
            max,
        } => {
            if let Some(max) = max {
                if max < min {
                    return Err(verifier_err(
                        RULE_MANY_BOUNDS,
                        format!("{context}: invalid Many bounds min={min}, max={max}"),
                    ));
                }
            }
            verify_account_set_def(account_set, current, namespace_index, mode, context)
        }
        IdlAccountSetDef::Or(branches) => {
            if branches.is_empty() {
                return Err(verifier_err(
                    RULE_EMPTY_OR,
                    format!("{context}: Or account set cannot be empty"),
                ));
            }
            for (branch_index, branch) in branches.iter().enumerate() {
                verify_account_set_def(
                    branch,
                    current,
                    namespace_index,
                    mode,
                    &format!("{context} branch[{branch_index}]"),
                )?;
            }
            Ok(())
        }
    }
}

fn verify_type_def<'a>(
    type_def: &IdlTypeDef,
    current: &'a IdlDefinition,
    namespace_index: &NamespaceIndex<'a>,
    mode: VerificationMode,
    context: &str,
) -> Result<()> {
    match type_def {
        IdlTypeDef::Defined(type_id) => {
            verify_type_id(type_id, current, namespace_index, mode, context)?;
        }
        IdlTypeDef::FixedPoint { ty, .. } => {
            verify_type_def(ty, current, namespace_index, mode, context)?;
        }
        IdlTypeDef::Option { ty, .. } => {
            verify_type_def(ty, current, namespace_index, mode, context)?;
        }
        IdlTypeDef::List { len_ty, item_ty } => {
            verify_type_def(len_ty, current, namespace_index, mode, context)?;
            verify_type_def(item_ty, current, namespace_index, mode, context)?;
        }
        IdlTypeDef::UnsizedList {
            len_ty,
            offset_ty,
            item_ty,
        } => {
            verify_type_def(len_ty, current, namespace_index, mode, context)?;
            verify_type_def(offset_ty, current, namespace_index, mode, context)?;
            verify_type_def(item_ty, current, namespace_index, mode, context)?;
        }
        IdlTypeDef::Set { len_ty, item_ty } => {
            verify_type_def(len_ty, current, namespace_index, mode, context)?;
            verify_type_def(item_ty, current, namespace_index, mode, context)?;
        }
        IdlTypeDef::Map {
            len_ty,
            key_ty,
            value_ty,
        } => {
            verify_type_def(len_ty, current, namespace_index, mode, context)?;
            verify_type_def(key_ty, current, namespace_index, mode, context)?;
            verify_type_def(value_ty, current, namespace_index, mode, context)?;
        }
        IdlTypeDef::Array(inner, _) => {
            verify_type_def(inner, current, namespace_index, mode, context)?;
        }
        IdlTypeDef::Struct(fields) => {
            for (field_index, field) in fields.iter().enumerate() {
                verify_type_def(
                    &field.type_def,
                    current,
                    namespace_index,
                    mode,
                    &format!("{context} field[{field_index}]"),
                )?;
            }
        }
        IdlTypeDef::Enum { size, variants } => {
            verify_type_def(size, current, namespace_index, mode, context)?;
            for (variant_index, variant) in variants.iter().enumerate() {
                if let Some(variant_type) = &variant.type_def {
                    verify_type_def(
                        variant_type,
                        current,
                        namespace_index,
                        mode,
                        &format!("{context} variant[{variant_index}]"),
                    )?;
                }
            }
        }
        IdlTypeDef::Generic(_)
        | IdlTypeDef::Bool
        | IdlTypeDef::U8
        | IdlTypeDef::I8
        | IdlTypeDef::U16
        | IdlTypeDef::I16
        | IdlTypeDef::U32
        | IdlTypeDef::I32
        | IdlTypeDef::F32
        | IdlTypeDef::U64
        | IdlTypeDef::I64
        | IdlTypeDef::F64
        | IdlTypeDef::U128
        | IdlTypeDef::I128
        | IdlTypeDef::String
        | IdlTypeDef::Pubkey
        | IdlTypeDef::RemainingBytes => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account::IdlAccount,
        account_set::{IdlAccountSet, IdlAccountSetDef, IdlAccountSetId},
        instruction::{IdlInstruction, IdlInstructionDef},
        ty::{IdlType, IdlTypeDef, IdlTypeId},
        CrateMetadata, IdlGeneric, IdlMetadata, ItemInfo, Version,
    };
    use solana_pubkey::Pubkey;

    fn base_definition(namespace: &str) -> IdlDefinition {
        IdlDefinition {
            address: Pubkey::new_unique(),
            metadata: IdlMetadata {
                crate_metadata: CrateMetadata {
                    name: namespace.to_string(),
                    version: Version::new(1, 0, 0),
                    ..CrateMetadata::default()
                },
                ..IdlMetadata::default()
            },
            ..IdlDefinition::default()
        }
    }

    fn item_info(name: &str, source: &str) -> ItemInfo {
        ItemInfo {
            name: name.to_string(),
            source: source.to_string(),
            description: vec![],
        }
    }

    fn type_id(source: &str) -> IdlTypeId {
        IdlTypeId {
            source: source.to_string(),
            namespace: None,
            provided_generics: vec![],
        }
    }

    fn insert_struct_type(definition: &mut IdlDefinition, source: &str) {
        definition.types.insert(
            source.to_string(),
            IdlType {
                info: item_info(source, source),
                generics: vec![],
                type_def: IdlTypeDef::Struct(vec![]),
            },
        );
    }

    fn insert_generic_struct_type(definition: &mut IdlDefinition, source: &str, count: usize) {
        definition.types.insert(
            source.to_string(),
            IdlType {
                info: item_info(source, source),
                generics: (0..count)
                    .map(|index| IdlGeneric {
                        name: format!("T{index}"),
                        description: String::new(),
                        generic_id: format!("T{index}"),
                    })
                    .collect(),
                type_def: IdlTypeDef::Struct(vec![]),
            },
        );
    }

    fn generic(index: usize) -> IdlGeneric {
        IdlGeneric {
            name: format!("T{index}"),
            description: String::new(),
            generic_id: format!("T{index}"),
        }
    }

    fn insert_account_set_with_generics(
        definition: &mut IdlDefinition,
        source: &str,
        type_generic_count: usize,
        account_generic_count: usize,
    ) {
        definition.account_sets.insert(
            source.to_string(),
            IdlAccountSet {
                info: item_info(source, source),
                type_generics: (0..type_generic_count).map(generic).collect(),
                account_generics: (0..account_generic_count).map(generic).collect(),
                account_set_def: IdlAccountSetDef::empty_struct(),
            },
        );
    }

    fn type_id_with_namespace(source: &str, namespace: &str) -> IdlTypeId {
        IdlTypeId {
            source: source.to_string(),
            namespace: Some(namespace.to_string()),
            provided_generics: vec![],
        }
    }

    fn insert_instruction(definition: &mut IdlDefinition, source: &str, type_source: &str) {
        definition.instructions.insert(
            source.to_string(),
            IdlInstruction {
                discriminant: vec![1],
                definition: IdlInstructionDef {
                    account_set: IdlAccountSetDef::empty_struct(),
                    type_id: type_id(type_source),
                },
            },
        );
    }

    fn assert_rule(result: Result<()>, rule: &str) {
        let error = result.expect_err("expected verifier to fail");
        let message = error.to_string();
        assert!(
            message.contains(rule),
            "expected error to contain `{rule}`, got `{message}`"
        );
    }

    #[test]
    fn valid_graph_passes() {
        let mut definition = base_definition("main_program");
        insert_struct_type(&mut definition, "Args");
        insert_instruction(&mut definition, "DoThing", "Args");
        verify_idl_definitions([&definition]).expect("expected valid graph");
    }

    #[test]
    fn duplicate_namespace_fails() {
        let definition_a = base_definition("dup");
        let definition_b = base_definition("dup");
        assert_rule(
            verify_idl_definitions([&definition_a, &definition_b]),
            RULE_DUPLICATE_NAMESPACE,
        );
    }

    #[test]
    fn empty_namespace_fails() {
        let definition = base_definition("");
        assert_rule(verify_idl_definitions([&definition]), RULE_EMPTY_NAMESPACE);
    }

    #[test]
    fn missing_type_fails() {
        let mut definition = base_definition("main_program");
        insert_instruction(&mut definition, "DoThing", "MissingType");
        assert_rule(verify_idl_definitions([&definition]), RULE_MISSING_TYPE);
    }

    #[test]
    fn missing_account_set_fails() {
        let mut definition = base_definition("main_program");
        insert_struct_type(&mut definition, "Args");
        definition.instructions.insert(
            "DoThing".to_string(),
            IdlInstruction {
                discriminant: vec![1],
                definition: IdlInstructionDef {
                    type_id: type_id("Args"),
                    account_set: IdlAccountSetDef::Defined(IdlAccountSetId {
                        source: "MissingSet".to_string(),
                        provided_type_generics: vec![],
                        provided_account_generics: vec![],
                    }),
                },
            },
        );
        assert_rule(
            verify_idl_definitions([&definition]),
            RULE_MISSING_ACCOUNT_SET,
        );
    }

    #[test]
    fn invalid_many_bounds_fail() {
        let mut definition = base_definition("main_program");
        insert_struct_type(&mut definition, "Args");
        definition.instructions.insert(
            "DoThing".to_string(),
            IdlInstruction {
                discriminant: vec![1],
                definition: IdlInstructionDef {
                    type_id: type_id("Args"),
                    account_set: IdlAccountSetDef::Many {
                        account_set: Box::new(IdlAccountSetDef::empty_struct()),
                        min: 2,
                        max: Some(1),
                    },
                },
            },
        );
        assert_rule(verify_idl_definitions([&definition]), RULE_MANY_BOUNDS);
    }

    #[test]
    fn empty_or_fails() {
        let mut definition = base_definition("main_program");
        insert_struct_type(&mut definition, "Args");
        definition.instructions.insert(
            "DoThing".to_string(),
            IdlInstruction {
                discriminant: vec![1],
                definition: IdlInstructionDef {
                    type_id: type_id("Args"),
                    account_set: IdlAccountSetDef::Or(vec![]),
                },
            },
        );
        assert_rule(verify_idl_definitions([&definition]), RULE_EMPTY_OR);
    }

    #[test]
    fn type_generic_arity_mismatch_fails() {
        let mut definition = base_definition("main_program");
        insert_generic_struct_type(&mut definition, "GenericType", 1);
        definition.instructions.insert(
            "DoThing".to_string(),
            IdlInstruction {
                discriminant: vec![1],
                definition: IdlInstructionDef {
                    type_id: IdlTypeId {
                        source: "GenericType".to_string(),
                        namespace: None,
                        provided_generics: vec![],
                    },
                    account_set: IdlAccountSetDef::empty_struct(),
                },
            },
        );
        assert_rule(
            verify_idl_definitions([&definition]),
            RULE_TYPE_GENERIC_ARITY,
        );
    }

    #[test]
    fn account_set_type_generic_arity_mismatch_fails() {
        let mut definition = base_definition("main_program");
        insert_struct_type(&mut definition, "Args");
        insert_account_set_with_generics(&mut definition, "GenericSet", 1, 0);
        definition.instructions.insert(
            "DoThing".to_string(),
            IdlInstruction {
                discriminant: vec![1],
                definition: IdlInstructionDef {
                    type_id: type_id("Args"),
                    account_set: IdlAccountSetDef::Defined(IdlAccountSetId {
                        source: "GenericSet".to_string(),
                        provided_type_generics: vec![],
                        provided_account_generics: vec![],
                    }),
                },
            },
        );
        assert_rule(
            verify_idl_definitions([&definition]),
            RULE_ACCOUNT_SET_TYPE_ARITY,
        );
    }

    #[test]
    fn account_set_account_generic_arity_mismatch_fails() {
        let mut definition = base_definition("main_program");
        insert_struct_type(&mut definition, "Args");
        insert_account_set_with_generics(&mut definition, "GenericSet", 0, 1);
        definition.instructions.insert(
            "DoThing".to_string(),
            IdlInstruction {
                discriminant: vec![1],
                definition: IdlInstructionDef {
                    type_id: type_id("Args"),
                    account_set: IdlAccountSetDef::Defined(IdlAccountSetId {
                        source: "GenericSet".to_string(),
                        provided_type_generics: vec![],
                        provided_account_generics: vec![],
                    }),
                },
            },
        );
        assert_rule(
            verify_idl_definitions([&definition]),
            RULE_ACCOUNT_SET_ACCOUNT_ARITY,
        );
    }

    #[test]
    fn missing_account_reference_fails() {
        let mut definition = base_definition("main_program");
        insert_struct_type(&mut definition, "Args");
        definition.account_sets.insert(
            "SetWithAccount".to_string(),
            IdlAccountSet {
                info: item_info("SetWithAccount", "SetWithAccount"),
                type_generics: vec![],
                account_generics: vec![],
                account_set_def: IdlAccountSetDef::Single(IdlSingleAccountSet {
                    program_accounts: vec![IdlAccountId {
                        namespace: None,
                        source: "MissingAccount".to_string(),
                    }],
                    ..IdlSingleAccountSet::default()
                }),
            },
        );
        definition.instructions.insert(
            "DoThing".to_string(),
            IdlInstruction {
                discriminant: vec![1],
                definition: IdlInstructionDef {
                    type_id: type_id("Args"),
                    account_set: IdlAccountSetDef::Defined(IdlAccountSetId {
                        source: "SetWithAccount".to_string(),
                        provided_type_generics: vec![],
                        provided_account_generics: vec![],
                    }),
                },
            },
        );
        assert_rule(verify_idl_definitions([&definition]), RULE_MISSING_ACCOUNT);
    }

    #[test]
    fn cross_namespace_missing_definition_fails() {
        let mut definition = base_definition("main_program");
        definition.accounts.insert(
            "MyAccount".to_string(),
            IdlAccount {
                discriminant: vec![7],
                type_id: type_id_with_namespace("ExternalType", "external_program"),
                seeds: None,
            },
        );
        assert_rule(
            verify_idl_definitions([&definition]),
            RULE_MISSING_NAMESPACE,
        );
    }

    #[test]
    fn compatibility_mode_allows_embedded_external_types() {
        let mut definition = base_definition("main_program");
        definition.external_types.insert(
            "ExternalType".to_string(),
            IdlType {
                info: item_info("ExternalType", "ExternalType"),
                generics: vec![],
                type_def: IdlTypeDef::Struct(vec![]),
            },
        );
        definition.accounts.insert(
            "MyAccount".to_string(),
            IdlAccount {
                discriminant: vec![7],
                type_id: type_id_with_namespace("ExternalType", "external_program"),
                seeds: None,
            },
        );
        verify_idl_definitions([&definition]).expect("expected compatibility mode to pass");
    }

    #[test]
    fn strict_mode_requires_namespaced_definition_set() {
        let mut definition = base_definition("main_program");
        definition.external_types.insert(
            "ExternalType".to_string(),
            IdlType {
                info: item_info("ExternalType", "ExternalType"),
                generics: vec![],
                type_def: IdlTypeDef::Struct(vec![]),
            },
        );
        definition.accounts.insert(
            "MyAccount".to_string(),
            IdlAccount {
                discriminant: vec![7],
                type_id: type_id_with_namespace("ExternalType", "external_program"),
                seeds: None,
            },
        );
        assert_rule(
            verify_idl_definitions_strict([&definition]),
            RULE_MISSING_NAMESPACE,
        );
    }

    #[test]
    fn strict_mode_accepts_when_external_definition_is_provided() {
        let mut main_definition = base_definition("main_program");
        main_definition.accounts.insert(
            "MyAccount".to_string(),
            IdlAccount {
                discriminant: vec![7],
                type_id: type_id_with_namespace("ExternalType", "external_program"),
                seeds: None,
            },
        );

        let mut external_definition = base_definition("external_program");
        insert_struct_type(&mut external_definition, "ExternalType");

        verify_idl_definitions_strict([&main_definition, &external_definition])
            .expect("expected strict mode to pass with full definition set");
    }
}
