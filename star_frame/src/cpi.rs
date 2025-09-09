//! Cross program invocation (CPI) builders and utilities.
use std::mem::MaybeUninit;

use crate::{
    account_set::{CpiAccountSet, DynamicCpiAccountSetLen},
    instruction::InstructionDiscriminant,
    prelude::*,
};
use borsh::object_length;
use bytemuck::bytes_of;
use itertools::Itertools;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{
        AccountMeta as PinocchioAccountMeta, Instruction as PinocchioInstruction,
        Seed as PinocchioSeed, Signer as PinocchioSigner,
    },
};
use typenum::{False, True};

/// A builder for creating a CPI instruction for a [`StarFrameProgram`].
///
/// Returned from [`MakeCpi::cpi`], and can be invoked with [`CpiBuilder::invoke`] or [`CpiBuilder::invoke_signed`].
#[must_use = "Did you forget to invoke the builder?"]
#[derive(Debug, Clone)]
pub struct CpiBuilder<'program, P, Ix, A>
where
    P: StarFrameProgram,
    Ix: BorshSerialize
        + StarFrameInstruction<Accounts<'static, 'static> = A>
        + InstructionDiscriminant<P::InstructionSet>,
    A: CpiAccountSet<AccountLen: HandleCpiArray, ContainsOption: CpiProgramInput<P>>,
{
    /// If the account set contains an option, the program [`AccountInfo`] must be passed in to the CPI builder.
    /// Otherwise, an [`Option<Pubkey>`] to override the program ID can be passed in.
    program: <A::ContainsOption as CpiProgramInput<P>>::Input<'program>,
    data: Ix,
    accounts: A::CpiAccounts,
}

/// Helper trait to handle the input to a CPI program.
///
/// When an account set contains an option, the program [`AccountInfo`] must be passed in to the CPI builder.
#[doc(hidden)]
pub trait CpiProgramInput<P: StarFrameProgram> {
    type Input<'a>: Clone + Debug + Copy;
    fn pubkey(input: Self::Input<'_>) -> &Pubkey;
    fn program(input: Self::Input<'_>) -> Option<&AccountInfo>;
}

#[allow(clippy::inline_always)]
impl<P: StarFrameProgram> CpiProgramInput<P> for False {
    type Input<'a> = Option<&'a Pubkey>;

    #[inline(always)]
    fn pubkey(input: Self::Input<'_>) -> &Pubkey {
        input.unwrap_or(&P::ID)
    }

    #[inline(always)]
    fn program(_input: Self::Input<'_>) -> Option<&AccountInfo> {
        None
    }
}

#[allow(clippy::inline_always)]
impl<P: StarFrameProgram> CpiProgramInput<P> for True {
    type Input<'a> = &'a AccountInfo;

    #[inline(always)]
    fn pubkey(input: Self::Input<'_>) -> &Pubkey {
        input.pubkey()
    }

    #[inline(always)]
    fn program(input: Self::Input<'_>) -> Option<&AccountInfo> {
        Some(input)
    }
}
/// Used to create a `CpiBuilder` for a [`StarFrameProgram`].
pub trait MakeCpi: StarFrameProgram + Sized {
    /// Creates a `CpiBuilder` with a `StarFrameInstruction`.
    ///
    /// - `program` - If the account set contains an `Option<T>`, the program's [`AccountInfo`] must be passed in to the CPI builder.
    ///   Otherwise, an [`Option<Pubkey>`] to override the program ID can be passed in.
    ///
    /// # Example
    /// ```ignore
    /// MyProgram::cpi(&MyInstruction { .. }, MyInstructionCpiAccounts { .. }, Some(&PROGRAM_ID_OVERRIDE))?.invoke()?;
    /// ```
    #[inline]
    fn cpi<I, A>(
        data: I,
        accounts: A::CpiAccounts,
        program: <A::ContainsOption as CpiProgramInput<Self>>::Input<'_>,
    ) -> CpiBuilder<'_, Self, I, A>
    where
        I: StarFrameInstruction<Accounts<'static, 'static> = A>
            + InstructionDiscriminant<Self::InstructionSet>
            + BorshSerialize,
        A: CpiAccountSet<AccountLen: HandleCpiArray, ContainsOption: CpiProgramInput<Self>>,
    {
        CpiBuilder {
            program,
            data,
            accounts,
        }
    }
}

impl<T> MakeCpi for T where T: StarFrameProgram + Sized {}

impl<P, Ix, A> CpiBuilder<'_, P, Ix, A>
where
    P: StarFrameProgram,
    Ix: BorshSerialize
        + StarFrameInstruction<Accounts<'static, 'static> = A>
        + InstructionDiscriminant<P::InstructionSet>,
    A: CpiAccountSet<AccountLen: HandleCpiArray, ContainsOption: CpiProgramInput<P>>,
{
    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn invoke(&self) -> Result<()> {
        self.invoke_signed(&[])
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> Result<()> {
        let mut infos_index = 0;
        let mut infos_arr = <<A as CpiAccountSet>::AccountLen as HandleCpiArray>::uninit_infos();
        A::write_account_infos(
            <A::ContainsOption as CpiProgramInput<P>>::program(self.program),
            &self.accounts,
            &mut infos_index,
            infos_arr.as_mut(),
        )?;

        let program_id = <A::ContainsOption as CpiProgramInput<P>>::pubkey(self.program);
        let mut metas_index = 0;
        let mut metas_arr = <<A as CpiAccountSet>::AccountLen as HandleCpiArray>::uninit_metas();
        A::write_account_metas(
            program_id,
            &self.accounts,
            &mut metas_index,
            metas_arr.as_mut(),
        );

        let nested_seeds: Vec<Vec<PinocchioSeed>> = signers_seeds
            .iter()
            .map(|seeds: &&[&[u8]]| {
                seeds
                    .iter()
                    .map(|seed| PinocchioSeed::from(*seed))
                    .collect_vec()
            })
            .collect_vec();
        let signers = nested_seeds
            .iter()
            .map(|seeds| seeds.as_slice().into())
            .collect_vec();

        let len = object_length(&self.data)?
            + size_of::<<<P as StarFrameProgram>::InstructionSet as InstructionSet>::Discriminant>(
            );
        let mut data = Vec::with_capacity(len);
        data.extend_from_slice(bytes_of(&Ix::DISCRIMINANT));
        self.data.serialize(&mut data)?;

        // SAFETY:
        // Our CpiAccountSet implementation ensures that the array has been initialized up to the index
        unsafe {
            <A as CpiAccountSet>::AccountLen::invoke_signed(
                program_id,
                data.as_slice(),
                infos_arr,
                infos_index,
                metas_arr,
                metas_index,
                &signers,
            )?;
        }

        Ok(())
    }
}
/// Private trait to handle CPI w/ fixed size arrays
#[doc(hidden)]
pub trait HandleCpiArray {
    type Arr<T>: AsMut<[MaybeUninit<T>]>;
    fn uninit_infos<'a>() -> Self::Arr<&'a AccountInfo>;
    fn uninit_metas<'a>() -> Self::Arr<PinocchioAccountMeta<'a>>;
    /// # Safety
    /// The metas and infos must be initialized up to their respective indices
    unsafe fn invoke_signed<'a>(
        program_id: &Pubkey,
        data: &[u8],
        infos: Self::Arr<&'a AccountInfo>,
        infos_index: usize,
        metas: Self::Arr<PinocchioAccountMeta<'a>>,
        metas_index: usize,
        signers: &[PinocchioSigner],
    ) -> Result<()>;
}

macro_rules! impl_handle_cpi_array {
    ($($n:tt)*) => {
        $(
            paste::paste! {
                impl HandleCpiArray for typenum::[<U $n>] {
                    type Arr<T> = [MaybeUninit<T>; $n];
                    #[inline(always)]
                    fn uninit_infos<'a>() -> Self::Arr<&'a AccountInfo> {
                        [const { MaybeUninit::uninit() }; $n]

                    }
                    #[inline(always)]
                    fn uninit_metas<'a>() -> Self::Arr<PinocchioAccountMeta<'a>> {
                        // SAFETY:
                        // This is fine because it's initializing an array of uninits.
                        // For some reason this is more effecient than a const init?
                        unsafe { MaybeUninit::uninit().assume_init() }
                    }
                    #[inline(always)]
                    unsafe fn invoke_signed<'a>(
                        program_id: &Pubkey,
                        data: &[u8],
                        infos: Self::Arr<&'a AccountInfo>,
                        infos_index: usize,
                        metas: Self::Arr<PinocchioAccountMeta<'a>>,
                        metas_index: usize,
                        signers: &[PinocchioSigner],
                    ) -> Result<()> {
                        assert_eq!(infos_index, infos.len());
                        assert_eq!(metas_index, metas.len());

                        let metas: [PinocchioAccountMeta<'a>; $n] = metas.map(|meta|
                            // SAFETY:
                            // We can assume the array has been initialized based on the index safety requirements
                            unsafe { meta.assume_init() }
                        );
                        let infos: [&AccountInfo; $n] = infos.map(|info|
                            // SAFETY:
                            // We can assume the array has been initialized based on the index safety requirements
                            unsafe { info.assume_init() }
                        );


                        pinocchio::cpi::invoke_signed(
                            &PinocchioInstruction {
                                program_id: program_id.as_array(),
                                data,
                                accounts:
                                &metas,
                            },
                            &infos,
                            signers,
                        )?;
                        Ok(())
                    }
                }
            }
        )*
    };
}

impl_handle_cpi_array!(
    0  1  2  3  4  5  6  7  8  9  10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
);

#[allow(clippy::inline_always)]
impl HandleCpiArray for DynamicCpiAccountSetLen {
    type Arr<T> = [MaybeUninit<T>; 64];

    #[inline(always)]
    fn uninit_infos<'a>() -> Self::Arr<&'a AccountInfo> {
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    #[inline(always)]
    fn uninit_metas<'a>() -> Self::Arr<PinocchioAccountMeta<'a>> {
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    #[inline(always)]
    unsafe fn invoke_signed<'a>(
        program_id: &Pubkey,
        data: &[u8],
        infos: Self::Arr<&'a AccountInfo>,
        infos_index: usize,
        metas: Self::Arr<PinocchioAccountMeta<'a>>,
        metas_index: usize,
        signers: &[PinocchioSigner],
    ) -> Result<()> {
        assert_eq!(infos_index, metas_index);

        let metas_slice = &metas[..metas_index];
        // SAFETY:
        // We can turn a slice of uninits to a slice of inits (we can assume up to the index is initialized)
        let metas_slice = unsafe {
            &*(std::ptr::from_ref::<[MaybeUninit<PinocchioAccountMeta<'a>>]>(metas_slice)
                as *const [PinocchioAccountMeta<'a>])
        };

        let infos_slice = &infos[..infos_index];
        // SAFETY:
        // We can turn a slice of uninits to a slice of inits (we can assume up to the index is initialized)
        let infos_slice = unsafe {
            &*std::ptr::from_ref::<[MaybeUninit<&AccountInfo>]>(infos_slice)
                .cast::<[&AccountInfo; 64]>()
        };

        pinocchio::cpi::slice_invoke_signed(
            &PinocchioInstruction {
                program_id: program_id.as_array(),
                data,
                accounts: metas_slice,
            },
            infos_slice,
            signers,
        )?;
        Ok(())
    }
}
