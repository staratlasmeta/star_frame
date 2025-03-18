use solana_program::msg;
use solana_program::program_error::ProgramError;

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn handle_error(error: anyhow::Error) -> ProgramError {
    if let Some(program_error) = error.downcast_ref::<ProgramError>() {
        msg!("{}", error);
        program_error.clone()
    } else {
        msg!("STAR FRAME ERROR: {}", error);
        ProgramError::Custom(426_000_000)
    }
}
