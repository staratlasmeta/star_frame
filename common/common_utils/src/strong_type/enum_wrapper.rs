use crate::UtilError;
use bytemuck::{Pod, Zeroable};
use common_utils::prelude::*;
use std::marker::PhantomData;

/// Trait for getting a unit enum value from its repr.
pub trait UnitEnumFromRepr: Copy {
    /// The repr of the enum.
    type Repr;
    /// Gets the enum value from its repr.
    fn from_repr(repr: Self::Repr) -> std::result::Result<Self, Self::Repr>;
    /// Gets the enum value from its repr, or returns an error.
    fn from_repr_or_error(repr: Self::Repr) -> Result<Self> {
        Self::from_repr(repr).map_err(|_| error!(UtilError::InvalidEnumDiscriminant))
    }
    /// Gets the repr of the enum value.
    fn into_repr(self) -> Self::Repr;
}

/// A [`Pod`] wrapper for a unit enum value.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct UnitEnumWrapper<E>
where
    E: UnitEnumFromRepr,
{
    value: E::Repr,
    enum_type: PhantomData<fn() -> E>,
}
impl<E> UnitEnumWrapper<E>
where
    E: UnitEnumFromRepr,
{
    /// Creates a new wrapper from the enum value.
    pub fn from_enum_value(value: E) -> Self {
        Self {
            value: value.into_repr(),
            enum_type: PhantomData,
        }
    }

    /// Gets the contained enum value.
    pub fn enum_value(self) -> Result<E> {
        E::from_repr_or_error(self.value)
    }

    /// Gets the contained enum value, or the contained value if it is not a valid enum value.
    pub fn enum_value_or_contained(self) -> std::result::Result<E, E::Repr> {
        E::from_repr(self.value)
    }
}
impl<E> From<E> for UnitEnumWrapper<E>
where
    E: UnitEnumFromRepr,
{
    fn from(value: E) -> Self {
        Self::from_enum_value(value)
    }
}
// Safety: This is a perfect implementation not provided by the derive macro.
unsafe impl<E> Zeroable for UnitEnumWrapper<E>
where
    E: UnitEnumFromRepr,
    E::Repr: Zeroable,
{
    fn zeroed() -> Self {
        Self {
            value: <E::Repr as Zeroable>::zeroed(),
            enum_type: PhantomData,
        }
    }
}
// Safety: This is a perfect implementation not provided by the derive macro.
unsafe impl<E> Pod for UnitEnumWrapper<E>
where
    E: UnitEnumFromRepr + 'static,
    E::Repr: Pod,
{
}
