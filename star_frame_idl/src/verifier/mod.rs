use crate::{IdlDefinition, Result};

pub fn verify_idl_definitions<'a, I>(_def_set: I) -> Result<()>
where
    I: IntoIterator,
    I::IntoIter: Iterator<Item = &'a IdlDefinition> + Clone,
{
    eprintln!("TODO: verify_idl_definitions");
    Ok(())
}
