mod cancel;
mod exchange;
mod init;

pub use cancel::*;
pub use exchange::*;
pub use init::*;

use star_frame::prelude::*;

#[star_frame_instruction_set]
pub enum EscrowInstructionSet {
    InitEscrow(InitEscrowIx),
    Cancel(CancelIx),
    Exchange(ExchangeIx),
}
