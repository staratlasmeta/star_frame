use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
};

use derive_more::{Deref, DerefMut, Display, Error as DeriveError};
use itertools::Itertools;
use pinocchio::{msg, program_error::ProgramError};
use pinocchio_log::{
    log,
    logger::{log_message, Logger},
};

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

#[must_use]
pub(crate) fn handle_error2(error: Error) -> ProgramError {
    error.log();
    error.into()
}

/// left is found, right is expected
#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr, $err:expr $(, $($reason:expr)?)?) => {{
        use $crate::errors::ErrorInfo;

        let left = $left;
        let right = $right;
        if left != right {
            return $crate::err!($err, $($($reason)*)*)
                .with_reason(|| format!("expected {:?}, found {:?}", right, left))
                .into();
        }
    }};
}

/// left is found, right is expected
#[macro_export]
macro_rules! ensure_ne {
    ($left:expr, $right:expr, $err:expr $(, $($reason:expr)?)?) => {{
        use $crate::errors::ErrorInfo;

        let left = $left;
        let right = $right;
        if left == right {
            return $crate::err!($err, $($($reason)*)*)
                .with_reason(|| format!("expected to not be {:?}, found {:?}", right, left))
                .into();
        }
    }};
}

/// Returns an Err<Error> if the condition is false
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr $(, $($reason:expr)?)?) => {
        if !$cond {
            return Err($crate::error!($err, $($($reason)*)*)).into();
        }
    };
}

/// Returns an Err<Error>
#[macro_export]
macro_rules! bail {
    ($err:expr $(, $($reason:expr)?)?) => {
        return $crate::err!($err, $($($reason)*)*);
    };
}

/// Construcs an Err<Error>
#[macro_export]
macro_rules! err {
    ($err:expr $(, $($reason:expr)?)?) => {
        Err($crate::error!($err, $($($reason)*)*))
    };
}

/// Constructs an Error
#[macro_export]
macro_rules! error {
    ($err:expr) => {
        $crate::errors::Error::new_with_source($err, $crate::star_frame_error_source!())
    };
    ($err:expr, $reason:expr) => {{
        $crate::error!($err).reason($reason)
    }};
}

// TODO: Make this a real thing
#[derive(Debug, derive_more::Display, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCode {
    InvalidArgument2 = 0,
    InvalidArgument = 1,
    InvalidInstructionData = 2,
    InvalidAccountData = 3,
    AccountDataTooSmall = 4,
    InsufficientFunds = 5,
}

pub trait StarFrameError: 'static + Debug {
    fn code(&self) -> u32;
    fn name(&self) -> Cow<'static, str>;
    fn message(&self) -> Cow<'static, str>;
}

impl StarFrameError for ErrorCode {
    fn code(&self) -> u32 {
        *self as u32
    }
    fn name(&self) -> Cow<'static, str> {
        match self {
            ErrorCode::InvalidArgument2 => "InvalidArgument2",
            ErrorCode::InvalidArgument => "InvalidArgument",
            ErrorCode::InvalidInstructionData => "InvalidInstructionData",
            ErrorCode::InvalidAccountData => "InvalidAccountData",
            ErrorCode::AccountDataTooSmall => "AccountDataTooSmall",
            ErrorCode::InsufficientFunds => "InsufficientFunds",
        }
        .into()
    }
    fn message(&self) -> Cow<'static, str> {
        match self {
            ErrorCode::InvalidArgument2 => "InvalidArgument2",
            ErrorCode::InvalidArgument => "InvalidArgument",
            ErrorCode::InvalidInstructionData => "InvalidInstructionData",
            ErrorCode::InvalidAccountData => "InvalidAccountData",
            ErrorCode::AccountDataTooSmall => "AccountDataTooSmall",
            ErrorCode::InsufficientFunds => "InsufficientFunds",
        }
        .into()
    }
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
    reasons: Vec<Cow<'static, str>>,
    #[error(not(source))]
    source: Option<ErrorSource>,
}

#[derive(Debug, DeriveError, Display, Deref, DerefMut)]
pub struct Error(#[error(source)] Box<ErrorInner>);

impl std::fmt::Display for ErrorInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.kind)?;
        if let Some(source) = self.source {
            writeln!(f, "\nOccurred at {source}")?;
        }
        if !self.account_path.is_empty() {
            writeln!(
                f,
                "\nFor account: {}",
                self.account_path.iter().rev().join(".")
            )?;
        }
        if !self.reasons.is_empty() {
            writeln!(f)?;
            for reason in &self.reasons {
                writeln!(f, "{reason}")?;
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

#[macro_export]
macro_rules! star_frame_error_source {
    () => {
        $crate::errors::ErrorSource {
            file: file!(),
            line: line!(),
        }
    };
}

impl From<borsh::io::Error> for ErrorKind {
    fn from(_error: borsh::io::Error) -> Self {
        ErrorKind::ProgramError(ProgramError::BorshIoError)
    }
}

impl<T> From<T> for Error
where
    T: Into<ErrorKind>,
{
    fn from(value: T) -> Self {
        Error::new(value)
    }
}

mod private {
    #[doc(hidden)]
    pub trait Sealed {}
    impl<T, E> Sealed for Result<T, E> where E: Into<super::Error> {}
}

pub trait ErrorInfo<T>: private::Sealed {
    /// Adds a reason to the error
    fn reason(self, reason: &'static str) -> Result<T, Error>;

    /// Add a reason to the error with a closure. The reason is evaluated lazily, and should be used when
    /// the reason is not static.
    fn with_reason<C>(self, with_reason: impl FnOnce() -> C) -> Result<T, Error>
    where
        C: Into<Cow<'static, str>>;

    /// Add an account path to the error, from the inner account name to outermost
    fn account_path(self, account_path: &'static str) -> Result<T, Error>;

    fn with_source(self, source: impl FnOnce() -> ErrorSource) -> Result<T, Error>;
}

impl<T, E> ErrorInfo<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn reason(self, reason: &'static str) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.into().reason(reason)),
        }
    }

    fn with_reason<C>(self, with_reason: impl FnOnce() -> C) -> Result<T, Error>
    where
        C: Into<Cow<'static, str>>,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.into().reason(with_reason())),
        }
    }

    fn account_path(self, account_path: &'static str) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.into().push_account_path(account_path)),
        }
    }

    fn with_source(self, source: impl FnOnce() -> ErrorSource) -> Result<T, Error> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(error.into().add_source(source())),
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
    pub fn new(error: impl CanMakeError) -> Self {
        Error(
            ErrorInner {
                kind: error.into(),
                account_path: vec![],
                reasons: vec![],
                source: None,
            }
            .into(),
        )
    }

    #[cold]
    #[must_use]
    pub fn new_with_source(error: impl CanMakeError, source: ErrorSource) -> Self {
        Error(
            ErrorInner {
                kind: error.into(),
                account_path: vec![],
                reasons: vec![],
                source: Some(source),
            }
            .into(),
        )
    }

    #[must_use]
    pub fn reason(mut self, reason: impl Into<Cow<'static, str>>) -> Self {
        self.reasons.push(reason.into());
        self
    }

    #[must_use]
    fn push_account_path(mut self, account_path: &'static str) -> Self {
        self.account_path.push(account_path);
        self
    }

    #[must_use]
    fn add_source(mut self, source: ErrorSource) -> Self {
        self.source = Some(source);
        self
    }

    pub fn log(&self) {
        match &self.kind {
            ErrorKind::ProgramError(program_error) => {
                log!("ProgramError: {}", program_error.to_string().as_str());
            }
            ErrorKind::Custom(custom) => {
                log!("Custom: {}", custom.name().as_ref());
            }
        }
        if let Some(source) = self.source {
            log!("Occurred at {}:{}", source.file, source.line);
        }
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
        for reason in &self.reasons {
            log_message(reason.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use eyre::Context as _;

    use super::*;

    #[test]
    fn test_error_display() {
        let error = Err::<(), _>(Error::new(ProgramError::IllegalOwner))
            .with_source(|| star_frame_error_source!())
            .reason("test")
            .account_path("key")
            .account_path("profiles")
            .account_path("thingy")
            .unwrap_err();
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
        // bail!(ProgramError::IllegalOwner);
        // ensure!(0 == 1, ProgramError::IllegalOwner, "Reason!!");
        let res = (|| {
            ensure_eq!(0, 1, ProgramError::IllegalOwner, "test");
            Ok(())
        })()
        .unwrap_err();
        res.log();
        println!("{res}");
        Ok(())
    }
}
