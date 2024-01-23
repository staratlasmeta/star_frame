pub mod account;
pub mod account_loader;
pub mod prelude;

use crate::account_set::SingleAccountSet;
use crate::sys_calls::SysCallInvoke;
use crate::Result;
use derivative::Derivative;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::{msg, system_instruction};

pub const ANCHOR_CLOSED_ACCOUNT_DISCRIMINATOR: [u8; 8] = [255, 255, 255, 255, 255, 255, 255, 255];

#[derive(Debug, Copy, Clone)]
#[must_use]
pub enum InitOrZeroed<'a, 'info> {
    Init {
        space: usize,
        funder: &'a AccountInfo<'info>,
        rent: Rent,
    },
    Zeroed,
}
impl<'a, 'info> InitOrZeroed<'a, 'info> {
    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init { .. })
    }

    pub fn is_zeroed(&self) -> bool {
        matches!(self, Self::Zeroed)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AnchorSeeds<'a> {
    pub seeds: &'a [&'a [u8]],
    pub bump: Option<u8>,
    pub program_id: Option<Pubkey>,
}

#[derive(Debug, Clone)]
pub struct ValidateReturn<'a, 'info> {
    pub init: Option<InitOrZeroed<'a, 'info>>,
    pub seeds: Option<(Vec<Vec<u8>>, Pubkey)>,
}

#[derive(Derivative, Default, Copy, Clone)]
#[derivative(Debug)]
pub struct AnchorValidateArgs<'a, 'info> {
    pub check_signer: bool,
    pub check_writable: bool,
    pub init: Option<InitOrZeroed<'a, 'info>>,
    pub seeds: Option<AnchorSeeds<'a>>,
    pub close: Option<&'a AccountInfo<'info>>,
}

impl<'a, 'info> AnchorValidateArgs<'a, 'info> {
    pub fn validate(
        self,
        account: &impl SingleAccountSet<'info>,
        sys_calls: &mut impl SysCallInvoke,
        discriminant: [u8; 8],
    ) -> Result<()> {
        if self.check_signer && !account.is_signer() {
            Err(ProgramError::MissingRequiredSignature)
        } else if self.check_writable && !account.is_writable() {
            Err(ProgramError::AccountBorrowFailed)
        } else {
            let seeds = match self.seeds {
                None => None,
                Some(seeds) => Some(match seeds.bump {
                    Some(bump) => {
                        let seeds_with_bump = seeds
                            .seeds
                            .iter()
                            .map(|s| s.to_vec())
                            .chain([vec![bump]])
                            .collect::<Vec<_>>();
                        let seeds_ref = seeds_with_bump
                            .iter()
                            .map(|s| s.as_slice())
                            .collect::<Vec<_>>();
                        let key = Pubkey::create_program_address(
                            &seeds_ref,
                            seeds
                                .program_id
                                .as_ref()
                                .unwrap_or(sys_calls.current_program_id()),
                        )?;
                        if &key != account.key() {
                            return Err(ProgramError::InvalidSeeds);
                        }
                        (seeds_with_bump, key)
                    }
                    None => {
                        let (key, bump) = Pubkey::find_program_address(
                            seeds.seeds,
                            seeds
                                .program_id
                                .as_ref()
                                .unwrap_or(sys_calls.current_program_id()),
                        );
                        if &key != account.key() {
                            return Err(ProgramError::InvalidSeeds);
                        }
                        let seeds_with_bump = seeds
                            .seeds
                            .iter()
                            .map(|s| s.to_vec())
                            .chain([vec![bump]])
                            .collect::<Vec<_>>();
                        (seeds_with_bump, key)
                    }
                }),
            };

            match self.init {
                None => Ok(()),
                Some(InitOrZeroed::Init {
                    space,
                    funder,
                    rent,
                }) => {
                    match seeds {
                        None => sys_calls.invoke(
                            &system_instruction::create_account(
                                funder.key,
                                account.key(),
                                rent.minimum_balance(space),
                                space as u64,
                                sys_calls.current_program_id(),
                            ),
                            &[funder.clone(), account.account_info_cloned()],
                        )?,
                        Some((seeds, program_id)) => {
                            if &program_id != sys_calls.current_program_id() {
                                msg!("Seeds program id mismatch");
                                return Err(ProgramError::InvalidSeeds);
                            }
                            sys_calls.invoke_signed(
                                &system_instruction::create_account(
                                    funder.key,
                                    account.key(),
                                    rent.minimum_balance(space),
                                    space as u64,
                                    sys_calls.current_program_id(),
                                ),
                                &[funder.clone(), account.account_info_cloned()],
                                &[&seeds.iter().map(|s| s.as_slice()).collect::<Vec<_>>()],
                            )?;
                        }
                    }
                    account.info_data_bytes_mut()?[..8].copy_from_slice(&discriminant);
                    Ok(())
                }
                Some(InitOrZeroed::Zeroed) => {
                    let mut account_data = account.info_data_bytes_mut()?;
                    for byte in account_data.iter() {
                        if *byte != 0 {
                            return Err(ProgramError::AccountAlreadyInitialized);
                        }
                    }
                    account_data[..8].copy_from_slice(&discriminant);
                    Ok(())
                }
            }
        }
    }
}

#[derive(Derivative, Default, Copy, Clone)]
#[derivative(Debug)]
pub struct AnchorCleanupArgs<'a, 'info> {
    pub close: Option<&'a AccountInfo<'info>>,
}

// impl<'a, 'info> AnchorCleanupArgs<'a, 'info> {
//     pub fn cleanup(
//         self,
//         account: &impl SingleAccountSet<'info>,
//         sys_calls: &mut impl SysCallInvoke,
//     ) -> Result<()> {
//         match self.close {
//             None => Ok(()),
//             Some(close) => {
//                 sys_calls.invoke(
//                     &system_instruction::transfer(
//                         account.key(),
//                         close.key,
//                         account.lamports(),
//                     ),
//                     &[account.account_info_cloned(), close.clone()],
//                 )?;
//                 Ok(())
//             }
//         }
//     }
// }
