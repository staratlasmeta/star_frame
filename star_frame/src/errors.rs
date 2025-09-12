use pinocchio::{msg, program_error::ProgramError};

// TODO: Replace Eyre with an error system similar to anchor's
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

/// left is found, right is expected
#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr, $message:tt) => {{
        use eyre::{eyre, WrapErr};

        let left = $left;
        let right = $right;
        if left != right {
            return Err(eyre!($message))
                .with_context(|| format!("expected {:?}, found {:?}", right, left))
                .into();
        }
    }};
}

/// left is found, right is expected
#[macro_export]
macro_rules! ensure_ne {
    ($left:expr, $right:expr, $message:tt) => {{
        use eyre::{eyre, WrapErr};

        let left = $left;
        let right = $right;
        if left != right {
            return Err(eyre!($message))
                .with_context(|| format!("expected to not be {:?}, found {:?}", right, left))
                .into();
        }
    }};
}
