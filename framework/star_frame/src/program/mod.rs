pub mod system_program;

use crate::instruction::InstructionSet;
use crate::program_account::ProgramAccount;
use crate::util::Network;
use crate::Result;
use bytemuck::Pod;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

/// A Solana program's definition.
pub trait Program {
    /// The instruction set used by this program.
    type InstructionSet<'a>: InstructionSet<'a>;
    /// The discriminant type used by this program's accounts.
    type Discriminant: Pod;

    /// Gets the program id.
    fn program_id() -> ProgramIds;
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
                .map(|id| *id),
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

/// An account registered to a program.
pub trait ProgramAccountEntry<A: ?Sized + ProgramAccount<OwnerProgram = Self>>: Program {}
