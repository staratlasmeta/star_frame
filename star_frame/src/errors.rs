use pinocchio::{msg, program_error::ProgramError};

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn handle_error(error: anyhow::Error) -> ProgramError {
    for (index, e) in error.chain().enumerate() {
        msg!("Error({}): {}", index, e);
    }
    if let Some(program_error) = error.downcast_ref::<ProgramError>() {
        *program_error
    } else {
        ProgramError::Custom(426_000_000)
    }
}
