use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
    panic::Location,
};

use derive_more::{Deref, DerefMut, Display, Error as DeriveError};
use itertools::Itertools;
use pinocchio::program_error::ProgramError;
use pinocchio_log::{log, logger::Logger};
pub use star_frame_proc::star_frame_error;

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_error(error: eyre::Error) -> ProgramError {
    for (index, e) in error.chain().enumerate() {
        msg!("Error({}): {}", index, e);
    }
    if let Some(program_error) = error.downcast_ref::<ProgramError>() {
        *program_error
    } else {
        ProgramError::Custom(426_000_000)
    }
}

// #[must_use]
// pub(crate) fn handle_error(error: Error) -> ProgramError {
//     error.log();
//     error.into()
// }

/// left is found, right is expected
#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr, $err:expr $(,)?) => {{
        let left = $left;
        let right = $right;
        if left != right {
            return $crate::err!($err, format!("expected {:?}, found {:?}", right, left)).into();
        }
    }};

    ($left:expr, $right:expr, $err:expr, $($ctx:tt)*) => {{
        if $left != $right {
            return $crate::err!($err, $($ctx)*).into();
        }
    }};
}

/// left is found, right is expected
#[macro_export]
macro_rules! ensure_ne {
    ($left:expr, $right:expr, $err:expr $(,)?) => {{
        let right = $right;
        if left == right {
            return $crate::err!(
                $err,
                format!("expected to not be {:?}, found {:?}", right, left)
            )
            .into();
        }
    }};
    ($left:expr, $right:expr, $err:expr, $($ctx:tt)*) => {{
        if $left == $right {
            return $crate::err!($err, $($ctx)*).into();
        }
    }};
}

/// Returns an Err<Error> if the condition is false
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr $(, $($ctx:tt)*)?) => {
        if !$cond {
            return Err($crate::error!($err, $($($ctx)*)*)).into();
        }
    };
}

/// Returns an Err<Error>
#[macro_export]
macro_rules! bail {
    ($err:expr $(, $($ctx:tt)*)?) => {
        return $crate::err!($err, $($($ctx)*)*)
    };
}

/// Construcs an Err<Error>
#[macro_export]
macro_rules! err {
    ($err:expr $(, $($ctx:tt)*)?) => {
        Err($crate::error!($err, $($($ctx)*)*))
    };
}

/// Constructs an Error
#[macro_export]
macro_rules! error {
    ($err:expr $(,)?) => {
        $crate::errors::Error::new($err)
    };
    ($err:expr, $($ctx:tt)*) => {
        $crate::errors::Error::new_with_ctx($err, format!($($ctx)*))
    };
}

#[star_frame_error]
pub enum ErrorCode {
    #[msg("An invalid argument was provided (the second)")]
    InvalidArgument2 = 0,
    #[msg("An invalid argument was provided")]
    InvalidArgument = 1,
    #[msg("An invalid instruction data was provided")]
    InvalidInstructionData = 2,
    #[msg("An invalid account data was provided")]
    InvalidAccountData = 3,
    #[msg("An account data was too small")]
    AccountDataTooSmall = 4,
    #[msg("Insufficient funds")]
    InsufficientFunds = 5,
    #[msg("TODO")]
    TODO = 6,
}

/// Represents something that can be used as a error.
///
/// Can be converted into an [`Error`] via [`From`].
///
/// Derivable on enums via [`derive@star_frame_error`].
pub trait StarFrameError: 'static + Debug {
    fn code(&self) -> u32;
    fn name(&self) -> Cow<'static, str>;
}

impl<T> From<T> for ErrorKind
where
    T: StarFrameError + 'static,
{
    fn from(error: T) -> Self {
        ErrorKind::Custom(Box::new(error))
    }
}

impl From<ProgramError> for ErrorKind {
    fn from(error: ProgramError) -> Self {
        ErrorKind::ProgramError(error)
    }
}

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

#[derive(Debug, DeriveError)]
pub struct ErrorInner {
    kind: ErrorKind,
    account_path: Vec<&'static str>,
    initial_ctx: Option<Cow<'static, str>>,
    initial_source: ErrorSource,
    ctxs: Vec<(ErrorSource, Cow<'static, str>)>,
}

#[derive(Debug, DeriveError, Display, Deref, DerefMut)]
pub struct Error(#[error(source)] Box<ErrorInner>);

impl std::fmt::Display for ErrorInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
        if !self.ctxs.is_empty() {
            for (source, ctx) in &self.ctxs {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[display("{}:{}", file, line)]
pub struct ErrorSource {
    file: &'static str,
    line: u32,
}

impl ErrorSource {
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

impl From<borsh::io::Error> for ErrorKind {
    fn from(_error: borsh::io::Error) -> Self {
        ErrorKind::ProgramError(ProgramError::BorshIoError)
    }
}

impl From<bytemuck::PodCastError> for ErrorKind {
    fn from(_error: bytemuck::PodCastError) -> Self {
        ErrorCode::InvalidInstructionData.into()
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

mod private {
    #[doc(hidden)]
    pub trait Sealed {}
    impl<T, E> Sealed for Result<T, E> where E: Into<super::Error> {}
}

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
    #[cold]
    #[must_use]
    #[track_caller]
    pub fn new(error: impl CanMakeError) -> Self {
        Self::new_inner(error, None, Location::caller())
    }

    #[cold]
    #[must_use]
    #[track_caller]
    pub fn new_with_ctx(error: impl CanMakeError, ctx: impl Into<Cow<'static, str>>) -> Self {
        Self::new_inner(error, Some(ctx.into()), Location::caller())
    }

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
                ctxs: vec![],
            }
            .into(),
        )
    }

    #[must_use]
    pub fn push_ctx(
        mut self,
        ctx: impl Into<Cow<'static, str>>,
        location: &'static Location<'static>,
    ) -> Self {
        // self.ctxs.push(ctx.into());
        self.ctxs.push((
            ErrorSource {
                file: location.file(),
                line: location.line(),
            },
            ctx.into(),
        ));
        self
    }

    #[must_use]
    fn push_account_path(mut self, account_path: &'static str) -> Self {
        self.account_path.push(account_path);
        self
    }

    pub fn log(&self) {
        {
            let mut logger = Logger::<300>::default();
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

        for (source, ctx) in &self.ctxs {
            log!("{}:{}: {}", source.file, source.line, ctx.as_ref());
        }
    }
}

#[cfg(test)]
mod tests {
    use eyre::Context as _;

    use super::*;

    #[test]
    fn test_error_display() {
        fn returns_error() -> Result<(), Error> {
            Err::<(), Error>(error!(ErrorCode::InvalidArgument, "AAAHGAHAHAH"))
                .ctx("test")
                .account_path("key")
                .account_path("profiles")
                .account_path("thingy")?;

            Err(ErrorCode::InvalidArgument)?;
            Ok(())
        }

        let error = returns_error().ctx("Outer ctx").unwrap_err();
        error.log();
        eprintln!();
        eprintln!("{error}");
    }

    // #[test]
    // fn test_return_error() -> Result<(), Box<Error>> {
    //     let error = Err::<(), _>(ErrorCode::InvalidArgument)?;
    //     Ok(())
    // }

    #[test]

    fn test_eyre_stuff() {
        let err = Err::<(), _>(eyre::eyre!("test"))
            .wrap_err("Hello")
            .wrap_err("World");
        eprintln!("{:#}", err.unwrap_err());
    }

    #[test]
    fn test_macros() -> Result<(), Error> {
        ensure!(0 == 1, ProgramError::IllegalOwner, "ctx!!");
        // bail!(ProgramError::IllegalOwner);
        let res = (|| {
            ensure_eq!(0, 1, ProgramError::IllegalOwner, "test");
            Ok(())
        })();

        let res = res.ctx("AAA").unwrap_err();

        res.log();
        println!("{res}");
        Ok(())
    }
}
