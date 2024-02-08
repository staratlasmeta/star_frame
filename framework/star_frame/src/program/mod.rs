pub mod system_program;

use crate::instruction::InstructionSet;
use crate::sys_calls::SysCallCore;
use crate::util::{compare_strings, Network};
use crate::Result;
use bytemuck::Pod;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
pub use star_frame_proc::program;

/// A Solana program's definition.
pub trait StarFrameProgram {
    /// The instruction set used by this program.
    type InstructionSet<'a>: InstructionSet<Discriminant = Self::InstructionDiscriminant>;
    type InstructionDiscriminant;

    type AccountDiscriminant: Pod + Eq;
    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant;

    const PROGRAM_IDS: ProgramIds;
    fn program_id(syscalls: &impl SysCallCore) -> Result<Pubkey> {
        Self::PROGRAM_IDS
            .find_network(syscalls.current_network())
            .map(|k| *k)
    }
}

#[must_use]
pub const fn search_for_network(program_ids: ProgramIds, network: Network) -> Option<Pubkey> {
    match program_ids {
        ProgramIds::Mapped(ids) => {
            let mut index = 0;
            loop {
                if index >= ids.len() {
                    break None;
                }

                let item_network = &ids[index];
                match (&item_network.0, &network) {
                    (Network::Mainnet, Network::Mainnet)
                    | (Network::Devnet, Network::Devnet)
                    | (Network::Testnet, Network::Testnet) => break Some(*item_network.1),
                    (Network::Custom(network1), Network::Custom(network2))
                        if compare_strings(network1, network2) =>
                    {
                        break Some(*item_network.1)
                    }
                    (_, _) => (),
                }

                index += 1;
            }
        }
        ProgramIds::AllNetworks(id) => Some(*id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compare_strings() {
        assert!(compare_strings("hello", "hello"));
        assert!(!compare_strings("hello", "world"));
        assert!(!compare_strings("hello", "hell"));
        assert!(!compare_strings("hello", "hellp"));
    }

    #[test]
    fn test_find_network() {
        const MAINNET_ID: Pubkey = Pubkey::new_from_array([0; 32]);
        const DEVNET_ID: Pubkey = Pubkey::new_from_array([1; 32]);
        const ATLASNET_ID: Pubkey = Pubkey::new_from_array([2; 32]);
        const PROGRAM_IDS: ProgramIds = ProgramIds::Mapped(&[
            (Network::Mainnet, &MAINNET_ID),
            (Network::Devnet, &DEVNET_ID),
            (Network::Custom("atlasnet"), &ATLASNET_ID),
        ]);
        assert_eq!(
            search_for_network(PROGRAM_IDS, Network::Mainnet),
            Some(MAINNET_ID)
        );
        assert_eq!(
            search_for_network(PROGRAM_IDS, Network::Devnet),
            Some(DEVNET_ID)
        );
        assert_eq!(
            search_for_network(PROGRAM_IDS, Network::Custom("atlasnet")),
            Some(ATLASNET_ID)
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProgramIds {
    Mapped(&'static [(Network, &'static Pubkey)]),
    AllNetworks(&'static Pubkey),
}
impl ProgramIds {
    pub fn find_network(&self, network: &Network) -> Result<&'static Pubkey> {
        match self {
            Self::Mapped(mapped) => mapped
                .iter()
                .find_map(|(net, id)| if net == network { Some(id) } else { None })
                .ok_or_else(|| {
                    msg!("Program not found for network: {:?}", network);
                    ProgramError::InvalidAccountData
                })
                .map(|id| *id)
                .map_err(Into::into),
            Self::AllNetworks(id) => Ok(id),
        }
    }
}
#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame_idl::NetworkKey;
    impl From<ProgramIds> for star_frame_idl::ProgramIds {
        fn from(value: ProgramIds) -> Self {
            match value {
                ProgramIds::Mapped(mapped) => star_frame_idl::ProgramIds::Mapped(
                    mapped
                        .iter()
                        .map(|(net, id)| {
                            (
                                (*net).into(),
                                NetworkKey {
                                    key: **id,
                                    extension_fields: Default::default(),
                                },
                            )
                        })
                        .collect(),
                ),
                ProgramIds::AllNetworks(id) => {
                    star_frame_idl::ProgramIds::AllNetworks(NetworkKey {
                        key: *id,
                        extension_fields: Default::default(),
                    })
                }
            }
        }
    }
}
