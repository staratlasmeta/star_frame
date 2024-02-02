use crate::instruction::InstructionSet;
use crate::program::StarFrameProgram;
use crate::util::Network;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
pub fn try_star_frame_entrypoint<T: StarFrameProgram>(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    network: Network,
) -> Result<()> {
    let mut syscalls = crate::sys_calls::solana_runtime::SolanaRuntime {
        program_id,
        network,
    };
    T::InstructionSet::handle_ix(instruction_data, program_id, accounts, &mut syscalls)
}

#[cfg(test)]
mod tests {
    use super::*;
    use star_frame_proc::{program, pubkey};
    // struct Stuff;
    impl StarFrameProgram for Stuff {
        type InstructionSet<'a> = ();
        type InstructionDiscriminant = ();
        type AccountDiscriminant = ();
        const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = ();
        const PROGRAM_IDS: ProgramIds = ProgramIds::Mapped(&[
            (
                Network::Mainnet,
                &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
            ),
            (
                Network::Custom("atlasnet"),
                &pubkey!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc"),
            ),
        ]);
    }

    use crate::program::ProgramIds;
    use star_frame::util::Network;

    #[program(Network::Mainnet, no_entrypoint)]
    pub struct Stuff;
}
