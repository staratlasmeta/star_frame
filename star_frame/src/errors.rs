use alloc::{borrow::Cow, boxed::Box, vec, vec::Vec};
use core::{
    fmt::{Debug, Formatter},
    panic::Location,
};

use alloc::string::ToString;
use derive_more::{Deref, DerefMut, Display, Error as DeriveError};
use itertools::Itertools;
use pinocchio::program_error::ProgramError;
use pinocchio_log::{log, logger::Logger};
pub use star_frame_proc::star_frame_error;

/// Error codes for errors emitted by `star_frame`
#[star_frame_error(offset = 0)]
pub enum ErrorCode {
    // Account set errors
    #[msg("Account is not writable")]
    ExpectedWritable = 1_000,
    #[msg("Account is not a signer")]
    ExpectedSigner,
    #[msg("Account's address does not match expected address")]
    AddressMismatch,
    #[msg("Discriminant mismatch")]
    DiscriminantMismatch,
    #[msg("Funder not set in account cache")]
    EmptyFunderCache,
    #[msg("Recipient not set in account cache")]
    EmptyRecipientCache,
    #[msg("Program not passed in for Optional account set")]
    MissingOptionalProgram,
    #[msg("Conflicting account seeds during init")]
    ConflictingAccountSeeds,
    #[msg("Seeds not set during init")]
    SeedsNotSet,

    // Unsized Type errors
    #[msg("An unexpected unsized type error occurred. This is a bug in star_frame")]
    UnsizedUnexpected = 2_000,
    #[msg("Pointer out of bounds in unsized type operation")]
    PointerOutOfBounds,
    #[msg("RawSliceAdvance out of bounds")]
    RawSliceAdvance,

    // Invalid input errors
    #[msg("Index out of bounds")]
    IndexOutOfBounds = 3_000,
    #[msg("Invalid range")]
    InvalidRange,

    // Conversion from other errors
    #[msg("num_traits::cast::ToPrimitive")]
    ToPrimitiveError = 9_000, // Conversion errors should be the last category
    #[msg("std::io::Error")]
    IoError,
    #[msg("bytemuck::PodCastError")]
    PodCastError,
    #[msg("bytemuck::checked::CheckedCastError")]
    CheckedCastError,
    #[msg("advancer::AdvanceError")]
    AdvanceError,
    #[msg("std::str::Utf8Error")]
    Utf8Error,
    #[msg("core::num::TryFromIntError")]
    TryFromIntError,
    #[msg("core::array::TryFromSliceError")]
    TryFromSliceError,
    #[msg("std::cell::BorrowError")]
    BorrowError,
    #[msg("std::cell::BorrowMutError")]
    BorrowMutError,
    #[msg("serde_json::Error")]
    SerdeJsonError,
    #[msg("star_frame_idl::Error")]
    IdlError,
}

/// Returns an [`Err<Error>`](Error) if left is not equal to right
///
/// left is found, right is expected
#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr, $err:expr $(,)?) => {{
        let left = $left;
        let right = $right;
        if left != right {
            return Err($crate::error!($err, "expected {right:?}, found {left:?}").into());
        }
    }};

    ($left:expr, $right:expr, $err:expr, $($ctx:tt)*) => {{
        if $left != $right {
            return Err($crate::error!($err, $($ctx)*).into());
        }
    }};
}

/// Returns an [`Err<Error>`](Error) if left is equal to right
///
/// left is found, right is expected
#[macro_export]
macro_rules! ensure_ne {
    ($left:expr, $right:expr, $err:expr $(,)?) => {{
        let right = $right;
        let left = $left;
        if left == right {
            return Err($crate::error!(
                $err,
                "expected to not be {right:?}, found {left:?}"
            ).into());
        }
    }};
    ($left:expr, $right:expr, $err:expr, $($ctx:tt)*) => {{
        if $left == $right {
            return Err($crate::error!($err, $($ctx)*).into());
        }
    }};
}

/// Returns an [`Err<Error>`](Error) if the condition is false
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr $(, $($ctx:tt)*)?) => {
        if !$cond {
            return Err($crate::error!($err, $($($ctx)*)*).into());
        }
    };
}

/// Returns an [`Err<Error>`](Error)
#[macro_export]
macro_rules! bail {
    ($err:expr $(, $($ctx:tt)*)?) => {
        return Err($crate::error!($err, $($($ctx)*)*).into())
    };
}

/// Constructs an [`Error`]
#[macro_export]
macro_rules! error {
    ($err:expr $(,)?) => {
        $crate::errors::Error::new($err)
    };
    ($err:expr, $($ctx:tt)*) => {
        $crate::errors::Error::new_with_ctx($err, $crate::alloc::format!($($ctx)*))
    };
}

/// Represents something that can be used as a error.
///
/// Can be converted into an [`Error`] via `From`.
///
/// Derivable on enums via [`macro@star_frame_error`].
pub trait StarFrameError: 'static + Debug + Send + Sync {
    fn code(&self) -> u32;
    fn name(&self) -> Cow<'static, str>;
}

/// The kind of error. Either a [`ProgramError`] or a custom error implementing [`StarFrameError`].
#[derive(Debug, Display)]
pub enum ErrorKind {
    #[display("ProgramError: {_0}")]
    ProgramError(ProgramError),
    #[display("StarFrameError: {}", _0.name())]
    Custom(Box<dyn StarFrameError + 'static>),
}

impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ProgramError(l0), Self::ProgramError(r0)) => l0 == r0,
            (Self::Custom(l0), Self::Custom(r0)) => l0.code() == r0.code(),
            _ => false,
        }
    }
}

/// The main body of the error struct, which is boxed to form [`Error`].
#[derive(Debug, DeriveError)]
pub struct ErrorInner {
    kind: ErrorKind,
    account_path: Vec<&'static str>,
    initial_ctx: Option<Cow<'static, str>>,
    initial_source: ErrorSource,
    context: Vec<(ErrorSource, Cow<'static, str>)>,
}

/// The error type returned from `star_frame` traits and functions.
#[derive(Debug, DeriveError, Display, Deref, DerefMut)]
pub struct Error(#[error(source)] Box<ErrorInner>);
static_assertions::assert_impl_all!(Error: Send, Sync);

impl core::fmt::Display for ErrorInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(initial_ctx) = &self.initial_ctx {
            write!(f, " - {initial_ctx}")?;
        }
        writeln!(f)?;

        writeln!(f, "Occurred at: {}", self.initial_source)?;

        if !self.account_path.is_empty() {
            writeln!(
                f,
                "For account: {}",
                self.account_path.iter().rev().join(".")
            )?;
        }
        if !self.context.is_empty() {
            for (source, ctx) in &self.context {
                writeln!(f, "{source}: {ctx}")?;
            }
        }
        Ok(())
    }
}

impl From<Error> for ProgramError {
    fn from(error: Error) -> Self {
        match &error.kind {
            ErrorKind::ProgramError(program_error) => *program_error,
            ErrorKind::Custom(custom) => ProgramError::Custom(custom.code()),
        }
    }
}

/// Where the error occurred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[display("{}:{}", file, line)]
pub struct ErrorSource {
    file: &'static str,
    line: u32,
}

impl ErrorSource {
    /// Creates a new error source for the caller's location
    #[track_caller]
    #[must_use]
    pub const fn new() -> Self {
        let location = Location::caller();
        Self {
            file: location.file(),
            line: location.line(),
        }
    }
}

impl Default for ErrorSource {
    fn default() -> Self {
        Self::new()
    }
}

mod private {
    #[doc(hidden)]
    pub trait Sealed {}
    impl<T, E> Sealed for Result<T, E> where E: Into<super::Error> {}
}

/// Adds additional context to an error, while automatically converting to the [`Error`] type.
pub trait ErrorInfo<T>: private::Sealed {
    /// Adds a ctx to the error
    #[track_caller]
    fn ctx(self, ctx: &'static str) -> Result<T, Error>;

    /// Add a ctx to the error with a closure. The ctx is evaluated lazily, and should be used when
    /// the ctx is not static.
    #[track_caller]
    fn with_ctx<C>(self, with_ctx: impl FnOnce() -> C) -> Result<T, Error>
    where
        C: Into<Cow<'static, str>>;

    /// Add an account path to the error, from the inner account name to outermost
    fn account_path(self, account_path: &'static str) -> Result<T, Error>;
}

impl<T, E> ErrorInfo<T> for Result<T, E>
where
    E: Into<Error>,
{
    #[track_caller]
    fn ctx(self, ctx: &'static str) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.into().push_ctx(ctx, Location::caller())),
        }
    }

    #[track_caller]
    fn with_ctx<C>(self, with_ctx: impl FnOnce() -> C) -> Result<T, Error>
    where
        C: Into<Cow<'static, str>>,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.into().push_ctx(with_ctx(), Location::caller())),
        }
    }

    fn account_path(self, account_path: &'static str) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.into().push_account_path(account_path)),
        }
    }
}

#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Errors in star_frame can only be made from types that implement StarFrameError or Into<ErrorKind>",
    note = "StarFrameError can be derived on enums with the #[star_frame_error] macro"
)]
pub trait CanMakeError: Into<ErrorKind> {}
impl<T> CanMakeError for T where T: Into<ErrorKind> {}

impl Error {
    /// Creates a new error at the caller's location
    #[cold]
    #[must_use]
    #[track_caller]
    pub fn new(error: impl CanMakeError) -> Self {
        Self::new_inner(error, None, Location::caller())
    }

    /// Creates a new error with additional context at the caller's location
    #[cold]
    #[must_use]
    #[track_caller]
    pub fn new_with_ctx(error: impl CanMakeError, ctx: impl Into<Cow<'static, str>>) -> Self {
        Self::new_inner(error, Some(ctx.into()), Location::caller())
    }

    #[cold]
    fn new_inner(
        error: impl CanMakeError,
        ctx: Option<Cow<'static, str>>,
        source: &'static Location<'static>,
    ) -> Self {
        Error(
            ErrorInner {
                kind: error.into(),
                account_path: vec![],
                initial_ctx: ctx,
                initial_source: ErrorSource {
                    file: source.file(),
                    line: source.line(),
                },
                context: vec![],
            }
            .into(),
        )
    }

    #[cold]
    #[must_use]
    fn push_ctx(
        mut self,
        ctx: impl Into<Cow<'static, str>>,
        location: &'static Location<'static>,
    ) -> Self {
        self.context.push((
            ErrorSource {
                file: location.file(),
                line: location.line(),
            },
            ctx.into(),
        ));
        self
    }

    #[cold]
    #[must_use]
    fn push_account_path(mut self, account_path: &'static str) -> Self {
        self.account_path.push(account_path);
        self
    }

    /// Logs the error using [`pinocchio_log`]
    pub fn log(&self) {
        {
            let mut logger = Logger::<1000>::default();
            match &self.kind {
                ErrorKind::ProgramError(program_error) => {
                    logger.append("ProgramError: ");
                    logger.append(program_error.to_string().as_str());
                }
                ErrorKind::Custom(custom) => {
                    logger.append("StarFrameError: ");
                    logger.append(custom.name().as_ref());
                }
            }
            if let Some(initial_ctx) = &self.initial_ctx {
                logger.append(" - ");
                logger.append(initial_ctx.as_ref());
            }
            logger.log();
        }

        log!(
            "Occurred at: {}:{}",
            self.initial_source.file,
            self.initial_source.line
        );

        if let Some((last, rest)) = self.account_path.split_last() {
            let mut logger = Logger::<200>::default();
            logger.append("For account: ");
            logger.append(*last);
            for account in rest.iter().rev() {
                logger.append(".");
                logger.append(*account);
            }
            logger.log();
        }

        for (source, ctx) in &self.context {
            log!(1000, "{}:{}: {}", source.file, source.line, ctx.as_ref(),);
        }
    }
}

// CONVERSIONS

impl<T> From<T> for ErrorKind
where
    T: StarFrameError + 'static,
{
    fn from(error: T) -> Self {
        ErrorKind::Custom(Box::new(error))
    }
}

impl<T> From<T> for Error
where
    T: Into<ErrorKind>,
{
    #[track_caller]
    fn from(value: T) -> Self {
        Error::new_inner(value, None, Location::caller())
    }
}

impl From<borsh::io::Error> for Error {
    #[track_caller]
    fn from(error: borsh::io::Error) -> Self {
        Error::new_inner(
            ErrorCode::IoError,
            Some(error.to_string().into()),
            Location::caller(),
        )
    }
}

impl From<bytemuck::PodCastError> for Error {
    #[track_caller]
    fn from(error: bytemuck::PodCastError) -> Self {
        Error::new_inner(
            ErrorCode::PodCastError,
            Some(error.to_string().into()),
            Location::caller(),
        )
    }
}

impl From<bytemuck::checked::CheckedCastError> for Error {
    #[track_caller]
    fn from(error: bytemuck::checked::CheckedCastError) -> Self {
        Error::new_inner(
            ErrorCode::CheckedCastError,
            Some(error.to_string().into()),
            Location::caller(),
        )
    }
}

impl From<advancer::AdvanceError> for Error {
    #[track_caller]
    fn from(error: advancer::AdvanceError) -> Self {
        Error::new_inner(
            ErrorCode::AdvanceError,
            Some(error.to_string().into()),
            Location::caller(),
        )
    }
}

impl From<core::str::Utf8Error> for Error {
    #[track_caller]
    fn from(error: core::str::Utf8Error) -> Self {
        Error::new_inner(
            ErrorCode::Utf8Error,
            Some(error.to_string().into()),
            Location::caller(),
        )
    }
}

impl From<core::array::TryFromSliceError> for Error {
    #[track_caller]
    fn from(error: core::array::TryFromSliceError) -> Self {
        Error::new_inner(
            ErrorCode::TryFromSliceError,
            Some(error.to_string().into()),
            Location::caller(),
        )
    }
}

// Static error messages with no useful extra information to log.

impl From<ProgramError> for ErrorKind {
    fn from(error: ProgramError) -> Self {
        ErrorKind::ProgramError(error)
    }
}

impl From<core::num::TryFromIntError> for ErrorKind {
    fn from(_error: core::num::TryFromIntError) -> Self {
        ErrorCode::TryFromIntError.into()
    }
}

impl From<core::cell::BorrowError> for ErrorKind {
    fn from(_error: core::cell::BorrowError) -> Self {
        ErrorCode::BorrowError.into()
    }
}

impl From<core::cell::BorrowMutError> for ErrorKind {
    fn from(_error: core::cell::BorrowMutError) -> Self {
        ErrorCode::BorrowMutError.into()
    }
}

impl From<solana_pubkey::PubkeyError> for ErrorKind {
    fn from(error: solana_pubkey::PubkeyError) -> Self {
        let program_error = match error {
            solana_pubkey::PubkeyError::MaxSeedLengthExceeded => {
                ProgramError::MaxSeedLengthExceeded
            }
            solana_pubkey::PubkeyError::InvalidSeeds => ProgramError::InvalidSeeds,
            solana_pubkey::PubkeyError::IllegalOwner => ProgramError::IllegalOwner,
        };
        ErrorKind::ProgramError(program_error)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impls {
    use super::*;
    impl From<star_frame_idl::Error> for Error {
        #[track_caller]
        fn from(error: star_frame_idl::Error) -> Self {
            Error::new_inner(
                ErrorCode::IdlError,
                Some(error.to_string().into()),
                Location::caller(),
            )
        }
    }

    impl From<serde_json::Error> for Error {
        #[track_caller]
        fn from(error: serde_json::Error) -> Self {
            Error::new_inner(
                ErrorCode::SerdeJsonError,
                Some(error.to_string().into()),
                Location::caller(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure() -> Result<(), Error> {
        ensure!(0 == 0, ProgramError::IllegalOwner, "Static str");
        ensure!(true, ProgramError::IllegalOwner);
        ensure!(true, ErrorCode::BorrowError, "Hello {}!", "world");
        let res: Result<(), Error> = (|| {
            ensure_eq!(0, 1, ProgramError::IllegalOwner, "Test {:?}", "aaa");
            ensure_eq!(0, 1, ProgramError::IllegalOwner);
            ensure_ne!(0, 1, ProgramError::IllegalOwner, "Test {}", "aaa");
            ensure_ne!(0, 1, ProgramError::IllegalOwner);
            Ok(())
        })();

        let res = res.ctx("AAA").unwrap_err();

        res.log();
        std::println!("{res}");
        Ok(())
    }

    #[test]
    fn test_bail() {
        let _: fn() -> Result<(), Error> = || bail!(ProgramError::IllegalOwner, "Static str");
        let _: fn() -> Result<(), Error> = || bail!(ProgramError::IllegalOwner);
        let _: fn() -> Result<(), Error> = || bail!(ErrorCode::BorrowError, "Hello {}!", "world");
    }
}
