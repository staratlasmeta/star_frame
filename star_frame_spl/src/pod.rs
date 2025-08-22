use star_frame::{
    align1::Align1,
    bytemuck::{Pod, Zeroable},
};

/// Duplicated from `spl_token_2022::PodCOption` to avoid a
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable, Align1)]
pub struct PodOption<T>
where
    T: Pod + Default,
{
    pub(crate) option: [u8; 4],
    pub(crate) value: T,
}
impl<T: Pod + Default> PodOption<T> {
    /// Represents that no value is stored in the option, like `Option::None`
    pub const NONE: [u8; 4] = [0; 4];
    /// Represents that some value is stored in the option, like
    /// `Option::Some(v)`
    pub const SOME: [u8; 4] = [1, 0, 0, 0];

    /// Create a PodCOption equivalent of `Option::None`
    ///
    /// This could be made `const` by using `std::mem::zeroed`, but that would
    /// require `unsafe` code, which is prohibited at the crate level.
    pub fn none() -> Self {
        Self {
            option: Self::NONE,
            value: T::default(),
        }
    }

    /// Create a PodCOption equivalent of `Option::Some(value)`
    pub const fn some(value: T) -> Self {
        Self {
            option: Self::SOME,
            value,
        }
    }

    /// Get the underlying value or another provided value if it isn't set,
    /// equivalent of `Option::unwrap_or`
    pub fn unwrap_or(self, default: T) -> T {
        if self.option == Self::NONE {
            default
        } else {
            self.value
        }
    }

    /// Checks to see if a value is set, equivalent of `Option::is_some`
    pub fn is_some(&self) -> bool {
        self.option == Self::SOME
    }

    /// Checks to see if no value is set, equivalent of `Option::is_none`
    pub fn is_none(&self) -> bool {
        self.option == Self::NONE
    }

    /// Converts the option into a Result, similar to `Option::ok_or`
    pub fn ok_or<E>(self, error: E) -> Result<T, E> {
        match self {
            Self {
                option: Self::SOME,
                value,
            } => Ok(value),
            _ => Err(error),
        }
    }

    /// Convenience wrapper around `Into<Option<T>>` for `PodOption<T>`
    pub fn into_option(self) -> Option<T> {
        self.into()
    }
}

impl<T> From<Option<T>> for PodOption<T>
where
    T: Pod + Default,
{
    fn from(option: Option<T>) -> Self {
        match option {
            Some(value) => Self::some(value),
            None => Self::none(),
        }
    }
}

impl<T> From<PodOption<T>> for Option<T>
where
    T: Pod + Default,
{
    fn from(option: PodOption<T>) -> Self {
        if option.is_some() {
            Some(option.value)
        } else {
            None
        }
    }
}
