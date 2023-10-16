use anchor_lang::error::{AnchorError, Error, ErrorCode, ProgramErrorWithOrigin};
use borsh::maybestd::io::Error as BorshIoError;
use solana_program::program_error::ProgramError;

/// A box wrapped anchor result
pub type Result<T, E = BoxedAnchorError> = std::result::Result<T, E>;

/// A box wrapped anchor error
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("{0}")]
pub struct BoxedAnchorError(#[source] Box<Error>);
impl BoxedAnchorError {
    /// Unwraps the inner error
    #[must_use]
    pub fn into_inner(self) -> Error {
        *self.0
    }
}
impl From<BoxedAnchorError> for Error {
    fn from(e: BoxedAnchorError) -> Self {
        e.into_inner()
    }
}
impl From<Error> for BoxedAnchorError {
    fn from(e: Error) -> Self {
        Self(Box::new(e))
    }
}
impl From<AnchorError> for BoxedAnchorError {
    fn from(ae: AnchorError) -> Self {
        Error::AnchorError(ae).into()
    }
}
impl From<ProgramError> for BoxedAnchorError {
    fn from(program_error: ProgramError) -> Self {
        Error::ProgramError(program_error.into()).into()
    }
}
impl From<BorshIoError> for BoxedAnchorError {
    fn from(error: BorshIoError) -> Self {
        Error::ProgramError(ProgramError::from(error).into()).into()
    }
}
impl From<ProgramErrorWithOrigin> for BoxedAnchorError {
    fn from(pe: ProgramErrorWithOrigin) -> Self {
        Error::ProgramError(pe).into()
    }
}
impl From<ErrorCode> for BoxedAnchorError {
    fn from(e: ErrorCode) -> Self {
        Error::from(e).into()
    }
}

/// A wrapper around anchor `error`
#[macro_export]
macro_rules! error {
    ($tt:expr) => {
        $crate::prelude::BoxedAnchorError::from($crate::anchor_lang::prelude::error!($tt))
    };
}

/// A wrapper around anchor `err`
#[macro_export]
macro_rules! err {
    ($error:tt $(,)?) => {
        Err($crate::error!(anchor_lang::ErrorCode::$error))
    };
    ($error:expr $(,)?) => {
        Err($crate::error!($error))
    };
}

/// A wrapper around anchor `require_gte`
#[macro_export]
macro_rules! require_gte {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 < $value2 {
            return Err($crate::anchor_lang::prelude::error!($error_code)
                .with_values(($value1, $value2))
                .into());
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 < $value2 {
            return Err($crate::anchor_lang::prelude::error!(
                anchor_lang::error::ErrorCode::RequireGteViolated
            )
            .with_values(($value1, $value2))
            .into());
        }
    };
}

/// A wrapper around anchor `require_gt`
#[macro_export]
macro_rules! require_gt {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 <= $value2 {
            return Err($crate::anchor_lang::prelude::error!($error_code)
                .with_values(($value1, $value2))
                .into());
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 <= $value2 {
            return Err($crate::anchor_lang::prelude::error!(
                anchor_lang::error::ErrorCode::RequireGtViolated
            )
            .with_values(($value1, $value2))
            .into());
        }
    };
}

/// A wrapper around anchor `require_keys_eq`
#[macro_export]
macro_rules! require_keys_eq {
    ($value1: expr, $value2: expr, $error_code:expr $(,)?) => {
        if $value1 != $value2 {
            return Err($crate::anchor_lang::prelude::error!($error_code)
                .with_pubkeys(($value1, $value2))
                .into());
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 != $value2 {
            return Err($crate::anchor_lang::prelude::error!(
                anchor_lang::error::ErrorCode::RequireKeysEqViolated
            )
            .with_pubkeys(($value1, $value2))
            .into());
        }
    };
}

/// A wrapper around anchor `require_eq`
#[macro_export]
macro_rules! require_eq {
    ($value1: expr, $value2: expr, $error_code:expr $(,)?) => {
        if $value1 != $value2 {
            return Err($crate::anchor_lang::error!($error_code)
                .with_values(($value1, $value2))
                .into());
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 != $value2 {
            return Err($crate::anchor_lang::error!(
                anchor_lang::error::ErrorCode::RequireEqViolated
            )
            .with_values(($value1, $value2))
            .into());
        }
    };
}

/// A wrapper around anchor `require`
#[macro_export]
macro_rules! require {
    ($invariant:expr, $error:tt $(,)?) => {
        if !($invariant) {
            return Err($crate::anchor_lang::error!(anchor_lang::ErrorCode::$error).into());
        }
    };
    ($invariant:expr, $error:expr $(,)?) => {
        if !($invariant) {
            return Err($crate::anchor_lang::error!($error).into());
        }
    };
}

/// A wrapper around anchor `require_keys_neq`
#[macro_export]
macro_rules! require_keys_neq {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 == $value2 {
            return Err($crate::anchor_lang::error!($error_code)
                .with_pubkeys(($value1, $value2))
                .into());
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 == $value2 {
            return Err($crate::anchor_lang::error!(
                $crate::anchor_lang::error::ErrorCode::RequireKeysNeqViolated
            )
            .with_pubkeys(($value1, $value2).into()));
        }
    };
}

/// A wrapper around anchor `require_neq`
#[macro_export]
macro_rules! require_neq {
    ($value1: expr, $value2: expr, $error_code: expr $(,)?) => {
        if $value1 == $value2 {
            return Err($crate::anchor_lang::error!($error_code)
                .with_values(($value1, $value2))
                .into());
        }
    };
    ($value1: expr, $value2: expr $(,)?) => {
        if $value1 == $value2 {
            return Err($crate::anchor_lang::error!(
                anchor_lang::error::ErrorCode::RequireNeqViolated
            )
            .with_values(($value1, $value2))
            .into());
        }
    };
}
