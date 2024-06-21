// #![allow(clippy::result_large_err)]

use bytemuck::Zeroable;
use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

// Declare the Program ID here to embed
// #[cfg_attr(feature = "prod", program(Network::Mainnet))]
#[program(Network::MainnetBeta)]
#[cfg_attr(
    feature = "atlasnet",
    program(star_frame::util::Network::Custom("atlasnet"))
)]
pub struct FactionEnlistment;

impl StarFrameProgram for FactionEnlistment {
    type InstructionSet<'a> = FactionEnlistmentInstructionSet<'a>;
    type InstructionDiscriminant = u8;

    type AccountDiscriminant = [u8; 8];

    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = [u8::MAX; 8];
    const PROGRAM_IDS: ProgramIds = ProgramIds::Mapped(&[
        (
            Network::MainnetBeta,
            &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
        ),
        (
            Network::Devnet,
            &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
        ),
        (
            Network::Localhost,
            &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
        ),
        (
            Network::Custom("atlasnet"),
            &pubkey!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc"),
        ),
    ]);
}

#[instruction_set2]
pub enum FactionEnlistmentInstructionSet {
    ProcessEnlistPlayer(ProcessEnlistPlayerIx),
}

#[derive(
    Copy, Clone, Zeroable, Align1, CheckedBitPattern, NoUninit, BorshDeserialize, BorshSerialize,
)]
#[borsh(crate = "borsh")]
#[repr(C, packed)]
pub struct ProcessEnlistPlayerIx {
    bump: u8,
    faction_id: FactionId,
}

impl FrameworkInstruction for ProcessEnlistPlayerIx {
    type SelfData<'a> = Self;

    type DecodeArg<'a> = ();
    type ValidateArg<'a> = u8;
    type RunArg<'a> = FactionId;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = ProcessEnlistPlayer<'info>
        where 'info: 'b;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        Self::deserialize(bytes).map_err(Into::into)
    }

    fn split_to_args<'a>(
        r: &'a Self::SelfData<'_>,
    ) -> (
        Self::DecodeArg<'a>,
        Self::ValidateArg<'a>,
        Self::RunArg<'a>,
        Self::CleanupArg<'a>,
    ) {
        ((), r.bump, r.faction_id, ())
    }

    fn run_instruction<'b, 'info>(
        faction_id: Self::RunArg<'_>,
        _program_id: &Pubkey,
        account_set: &mut Self::Accounts<'b, '_, 'info>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
        let check1 = sol_remaining_compute_units();
        let mut combined = account_set.player_faction_account.data_mut()?;
        let after_data = sol_remaining_compute_units();
        combined.sized1 = true;
        combined.sized2 = 69.into();
        combined.sized3 = 4;
        let after_set = sol_remaining_compute_units();
        let mut list2 = (&mut combined).list2()?;
        let list2_deser = sol_remaining_compute_units();
        list2.push(TestStruct { val1: 1, val2: 0 })?;

        let after_list2 = sol_remaining_compute_units();
        let mut list1 = (&mut combined).list1()?;
        let list1_deser = sol_remaining_compute_units();
        list1.push_all([10, 20, 30, 40, 50, 60, 70, 199])?;
        let after_push = sol_remaining_compute_units();
        let mut other = (&mut combined).other()?;
        let deser_other = sol_remaining_compute_units();

        (&mut other).list1()?.push(10)?;
        (&mut other)
            .list2()?
            .push(TestStruct { val1: 1, val2: 0 })?;
        let end = sol_remaining_compute_units();
        msg!("Initial data deser: {}", check1 - after_data - 100);
        msg!("Data set: {}", after_data - after_set - 100);
        msg!("List2 deser: {}", after_set - list2_deser - 100);
        msg!("List2 push: {}", list2_deser - after_list2 - 100);
        msg!("List1 deser: {}", after_list2 - list1_deser - 100);
        msg!("List1 push: {}", list1_deser - after_push - 100);
        msg!("Other deser: {}", after_push - deser_other - 100);
        msg!("Other push: {}", deser_other - end - 100);
        Ok(())
    }
}
pub use combined_test_3_impls::*;
pub use star_frame::serialize::unsize::test::CombinedTestExt;

// let clock = sys_calls.get_clock()?;
//
// let test_struct = TestStruct { val1: 0, val2: 0 };
// msg!("About to access data mut");
// let player_faction_account = &mut account_set.player_faction_account;
// let before = sol_remaining_compute_units();
// let mut data_mut = player_faction_account.data_mut()?;
// let mut list2 = (&mut data_mut).list2()?;
// list2.push_all([
// test_struct,
// test_struct,
// test_struct,
// test_struct,
// test_struct,
// ])?;
// let mut list1 = (&mut data_mut).list1()?;
// list1.push(0)?;
//
// // sol_log_compute_units();
// // let bump = account_set.player_faction_account.access_seeds().bump;
// // *account_set.player_faction_account.data_mut()? = PlayerFactionData {
// //     owner: *account_set.player_account.key,
// //     enlisted_at_timestamp: clock.unix_timestamp,
// //     faction_id,
// //     bump,
// //     _padding: [0; 5],
// // };
// Ok(())

#[derive(AccountSet)]
#[validate(arg = u8)]
#[account_set(skip_default_idl)]
pub struct ProcessEnlistPlayer<'info> {
    /// The player faction account
    #[validate(
        arg = Create(SeededInit {
            seeds: PlayerFactionAccountSeeds {
                player_account: *self.player_account.key()
            },
            init_create: CreateAccount::new(
                &self.system_program,
                &self.player_account,
            )
        })
    )]
    #[cleanup(arg = NormalizeRent {
        system_program: &self.system_program,
        funder: &self.player_account,
    })]
    pub player_faction_account: SeededInitAccount<'info, CombinedTest3>,
    /// The player account
    pub player_account: Writable<Signer<SystemAccount<'info>>>,
    /// Solana System program
    pub system_program: Program<'info, SystemProgram>,
}
#[derive(
    Debug,
    Align1,
    Copy,
    Clone,
    CheckedBitPattern,
    NoUninit,
    Eq,
    PartialEq,
    Zeroable, /*TypeToIdl, AccountToIdl*/
)]
#[repr(C, packed)]
// #[account(seeds = PlayerFactionAccountSeeds)]
pub struct PlayerFactionData {
    pub owner: Pubkey,
    pub enlisted_at_timestamp: bool,
    pub faction_id: FactionId,
    pub bump: u8,
    pub _padding: [u64; 5],
}

#[derive(
    Debug, Copy, Clone, CheckedBitPattern, NoUninit, BorshDeserialize, BorshSerialize, Eq, PartialEq,
)]
#[borsh(crate = "borsh")]
#[repr(u8)]
pub enum FactionId {
    MUD,
    ONI,
    Ustur,
}

unsafe impl Zeroable for FactionId {}

// TODO - Macro should derive this and with the idl feature enabled would also derive `AccountToIdl` and `TypeToIdl`
impl ProgramAccount for CombinedTest3 {
    type OwnerProgram = StarFrameDeclaredProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant =
        [47, 44, 255, 15, 103, 77, 139, 247];
}

impl SeededAccountData for CombinedTest3 {
    type Seeds = PlayerFactionAccountSeeds;
}

#[derive(Debug)]
pub struct PlayerFactionAccountSeeds {
    // #[constant(FACTION_ENLISTMENT)]
    player_account: Pubkey,
}

// TODO - Macro this
impl GetSeeds for PlayerFactionAccountSeeds {
    fn seeds(&self) -> Vec<&[u8]> {
        vec![b"FACTION_ENLISTMENT".as_ref(), self.player_account.seed()]
    }
}

use star_frame::prelude::CombinedRef;
use star_frame::prelude::*;
use star_frame::prelude::{
    CombinedExt, CombinedTRef, CombinedURef, CombinedUnsized, List, UnsizedInit, UnsizedType,
};
use star_frame::serialize::ref_wrapper::{
    AsBytes, AsMutBytes, RefBytes, RefBytesMut, RefResize, RefWrapper, RefWrapperMutExt,
    RefWrapperTypes,
};
use star_frame::serialize::unsize::resize::Resize;
use star_frame::serialize::unsize::test::CombinedTest;
use star_frame::serialize::unsize::test::TestStruct;
use star_frame::serialize::unsize::FromBytesReturn;
// #[unsized_type]

#[derive(Debug, Copy, Clone, CheckedBitPattern, Zeroable, Align1, NoUninit, PartialEq, Eq)]
#[repr(C, packed)]
pub struct CombinedTest3Sized {
    pub sized1: bool,
    pub sized2: PackedValue<u16>,
    pub sized3: u8,
}

pub use combined_test_3_impls::*;
use star_frame::serialize::list::ListExt;
use star_frame::solana_program::compute_units::sol_remaining_compute_units;

mod combined_test_3_impls {
    use super::*;
    use bytemuck::checked::{try_from_bytes, try_from_bytes_mut};
    use star_frame::serialize::ref_wrapper::{RefDeref, RefDerefMut};
    use star_frame::serialize::unsize::init::Zeroed;
    use std::ops::{Deref, DerefMut};

    type SizedField = CombinedTest3Sized;
    type Field1 = List<u8>;
    type Field2 = List<TestStruct>;
    type Field3 = CombinedTest;

    #[derive(Debug, Align1)]
    #[repr(transparent)]
    pub struct CombinedTest3(CombinedTest3Inner);

    type CombinedTest3Inner =
        CombinedUnsized<SizedField, CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>>;

    #[derive(Debug, Copy, Clone)]
    #[repr(transparent)]
    pub struct CombinedTest3Meta(<CombinedTest3Inner as UnsizedType>::RefMeta);

    // TODO: Where clause for derives?
    #[derive(Debug, Copy, Clone)]
    #[repr(transparent)]
    pub struct CombinedTest3Ref(<CombinedTest3Inner as UnsizedType>::RefData);

    pub struct CombinedTest3Owned {
        sized_struct: <SizedField as UnsizedType>::Owned,
        pub list1: <Field1 as UnsizedType>::Owned,
        pub list2: <Field2 as UnsizedType>::Owned,
        pub other: <Field3 as UnsizedType>::Owned,
    }

    impl Deref for CombinedTest3Owned {
        type Target = SizedField;
        fn deref(&self) -> &Self::Target {
            &self.sized_struct
        }
    }

    impl DerefMut for CombinedTest3Owned {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.sized_struct
        }
    }

    unsafe impl UnsizedType for CombinedTest3 {
        type RefMeta = CombinedTest3Meta;
        type RefData = CombinedTest3Ref;
        type Owned = CombinedTest3Owned;
        type IsUnsized = <CombinedTest3Inner as UnsizedType>::IsUnsized;

        unsafe fn from_bytes<S: AsBytes>(
            super_ref: S,
        ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
            unsafe {
                Ok(<CombinedTest3Inner as UnsizedType>::from_bytes(super_ref)?
                    .map_ref(|_, r| CombinedTest3Ref(r))
                    .map_meta(CombinedTest3Meta))
            }
        }

        fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> anyhow::Result<Self::Owned> {
            let (sized_struct, (list1, (list2, other))) =
                <CombinedTest3Inner as UnsizedType>::owned(unsafe { r.wrap_r(|_, r| r.0) })?;
            Ok(CombinedTest3Owned {
                sized_struct,
                list1,
                list2,
                other,
            })
        }
    }

    // CombinedUnsized<CombinedTest3Sized, CombinedUnsized<<List<u8>, CombinedUnsized<List<TestStruct>, CombinedTest>>>
    pub struct CombinedTest3Init<SizedStruct, List1, List2, Other> {
        pub sized_struct: SizedStruct,
        pub list1: List1,
        pub list2: List2,
        pub other: Other,
    }
    impl<SizedStruct, List1, List2, Other>
        UnsizedInit<CombinedTest3Init<SizedStruct, List1, List2, Other>> for CombinedTest3
    where
        SizedField: UnsizedInit<SizedStruct>,
        Field1: UnsizedInit<List1>,
        Field2: UnsizedInit<List2>,
        Field3: UnsizedInit<Other>,
    {
        const INIT_BYTES: usize =
            <CombinedTest3Inner as UnsizedInit<(SizedStruct, (List1, (List2, Other)))>>::INIT_BYTES;

        unsafe fn init<S: AsMutBytes>(
            super_ref: S,
            arg: CombinedTest3Init<SizedStruct, List1, List2, Other>,
        ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
            unsafe {
                let (r, m) = <CombinedTest3Inner as UnsizedInit<(
                    SizedStruct,
                    (List1, (List2, Other)),
                )>>::init(
                    super_ref,
                    (arg.sized_struct, (arg.list1, (arg.list2, arg.other))),
                )?;
                Ok((r.wrap_r(|_, r| CombinedTest3Ref(r)), CombinedTest3Meta(m)))
            }
        }
    }

    impl UnsizedInit<Zeroed> for CombinedTest3
    where
        SizedField: UnsizedInit<Zeroed>,
        Field1: UnsizedInit<Zeroed>,
        Field2: UnsizedInit<Zeroed>,
        Field3: UnsizedInit<Zeroed>,
    {
        const INIT_BYTES: usize = <CombinedTest3Inner as UnsizedInit<Zeroed>>::INIT_BYTES;

        unsafe fn init<S: AsMutBytes>(
            super_ref: S,
            arg: Zeroed,
        ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
            unsafe {
                let (r, m) = <CombinedTest3Inner as UnsizedInit<Zeroed>>::init(super_ref, arg)?;
                Ok((r.wrap_r(|_, r| CombinedTest3Ref(r)), CombinedTest3Meta(m)))
            }
        }
    }

    unsafe impl<S> RefBytes<S> for CombinedTest3Ref
    where
        S: AsBytes,
    {
        fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
            wrapper.sup().as_bytes()
        }
    }
    unsafe impl<S> RefBytesMut<S> for CombinedTest3Ref
    where
        S: AsMutBytes,
    {
        fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
            unsafe { wrapper.sup_mut().as_mut_bytes() }
        }
    }
    unsafe impl<S> RefResize<S, <CombinedTest3Inner as UnsizedType>::RefMeta> for CombinedTest3Ref
    where
        S: Resize<CombinedTest3Meta>,
    {
        unsafe fn resize(
            wrapper: &mut RefWrapper<S, Self>,
            new_byte_len: usize,
            new_meta: <CombinedTest3Inner as UnsizedType>::RefMeta,
        ) -> anyhow::Result<()> {
            unsafe {
                wrapper.r_mut().0 = CombinedRef::new(new_meta);
                wrapper
                    .sup_mut()
                    .resize(new_byte_len, CombinedTest3Meta(new_meta))
            }
        }

        unsafe fn set_meta(
            wrapper: &mut RefWrapper<S, Self>,
            new_meta: <CombinedTest3Inner as UnsizedType>::RefMeta,
        ) -> anyhow::Result<()> {
            unsafe {
                wrapper.r_mut().0 = CombinedRef::new(new_meta);
                wrapper.sup_mut().set_meta(CombinedTest3Meta(new_meta))
            }
        }
    }

    impl<S> RefDeref<S> for CombinedTest3Ref
    where
        S: AsBytes,
    {
        type Target = CombinedTest3Sized;
        fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target {
            let bytes = wrapper.sup().as_bytes().expect("Invalid bytes");
            try_from_bytes(&bytes[0..core::mem::size_of::<Self::Target>()]).expect("Invalid bytes")
        }
    }

    impl<S> RefDerefMut<S> for CombinedTest3Ref
    where
        S: AsMutBytes,
    {
        fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target {
            let bytes = unsafe { wrapper.sup_mut() }
                .as_mut_bytes()
                .expect("Invalid bytes");
            try_from_bytes_mut(&mut bytes[0..core::mem::size_of::<Self::Target>()])
                .expect("Invalid bytes")
        }
    }

    type CombinedTest3RefInner =
        CombinedRef<SizedField, CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>>;

    type SizedStruct<S> = RefWrapper<
        RefWrapper<
            RefWrapper<S, CombinedTest3RefInner>,
            CombinedTRef<SizedField, CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>>,
        >,
        <SizedField as UnsizedType>::RefData,
    >;

    type List1<S> = RefWrapper<
        RefWrapper<
            RefWrapper<
                RefWrapper<
                    RefWrapper<S, CombinedTest3RefInner>,
                    CombinedURef<
                        SizedField,
                        CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                    >,
                >,
                CombinedRef<Field1, CombinedUnsized<Field2, Field3>>,
            >,
            CombinedTRef<Field1, CombinedUnsized<Field2, Field3>>,
        >,
        <Field1 as UnsizedType>::RefData,
    >;

    type List2<S> = RefWrapper<
        RefWrapper<
            RefWrapper<
                RefWrapper<
                    RefWrapper<
                        RefWrapper<
                            RefWrapper<
                                S,
                                CombinedRef<
                                    SizedField,
                                    CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                                >,
                            >,
                            CombinedURef<
                                SizedField,
                                CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                            >,
                        >,
                        CombinedRef<Field1, CombinedUnsized<Field2, Field3>>,
                    >,
                    CombinedURef<Field1, CombinedUnsized<Field2, Field3>>,
                >,
                CombinedRef<Field2, Field3>,
            >,
            CombinedTRef<Field2, Field3>,
        >,
        <Field2 as UnsizedType>::RefData,
    >;

    type Other<S> = RefWrapper<
        RefWrapper<
            RefWrapper<
                RefWrapper<
                    RefWrapper<
                        RefWrapper<
                            RefWrapper<
                                S,
                                CombinedRef<
                                    SizedField,
                                    CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                                >,
                            >,
                            CombinedURef<
                                SizedField,
                                CombinedUnsized<Field1, CombinedUnsized<Field2, Field3>>,
                            >,
                        >,
                        CombinedRef<Field1, CombinedUnsized<Field2, Field3>>,
                    >,
                    CombinedURef<Field1, CombinedUnsized<Field2, Field3>>,
                >,
                CombinedRef<Field2, Field3>,
            >,
            CombinedURef<Field2, Field3>,
        >,
        <Field3 as UnsizedType>::RefData,
    >;

    pub trait CombinedTest3Ext: Sized + RefWrapperTypes {
        // fn sized_struct(self) -> anyhow::Result<SizedStruct<Self>>;
        fn list1(self) -> anyhow::Result<List1<Self>>;
        fn list2(self) -> anyhow::Result<List2<Self>>;
        fn other(self) -> anyhow::Result<Other<Self>>;
    }
    impl<R> CombinedTest3Ext for R
    where
        R: RefWrapperTypes<Ref = CombinedTest3Ref> + AsBytes,
    {
        // fn sized_struct(self) -> anyhow::Result<SizedStruct<Self>> {
        //     let r = self.r().0;
        //     unsafe { RefWrapper::new(self, r).t() }
        // }

        fn list1(self) -> anyhow::Result<List1<Self>> {
            let r = self.r().0;
            unsafe { RefWrapper::new(self, r).u()?.t() }
        }

        fn list2(self) -> anyhow::Result<List2<Self>> {
            let r = self.r().0;
            unsafe { RefWrapper::new(self, r).u()?.u()?.t() }
        }

        fn other(self) -> anyhow::Result<Other<Self>> {
            let r = self.r().0;
            unsafe { RefWrapper::new(self, r).u()?.u()?.u() }
        }
    }
}

// pub struct CombinedTest3 {
//     pub sized1: u8,
//     pub sized2: PackedValue<u16>,
//     pub sized3: bool,
//     #[unsized_start]
//     pub list1: List<u8>,
//     pub list2: List<TestStruct>,
//     pub other: CombinedTest,
// }

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::checked::try_from_bytes;
    use solana_client::rpc_config::{RpcSendTransactionConfig, RpcTransactionConfig};
    use solana_sdk::commitment_config::CommitmentConfig;
    use solana_sdk::native_token::LAMPORTS_PER_SOL;
    use solana_sdk::signature::{Keypair, Signer};
    use star_frame::solana_program::instruction::AccountMeta;

    #[tokio::test]
    async fn init_stuff() -> Result<()> {
        let client = solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
            "http://localhost:8899".to_string(),
            CommitmentConfig::confirmed(),
        );

        let player_account = Keypair::new();
        let res = client
            .request_airdrop(&player_account.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();
        client.poll_for_signature(&res).await.unwrap();

        let seeds = PlayerFactionAccountSeeds {
            player_account: player_account.pubkey(),
        };
        let (faction_account, bump) = Pubkey::find_program_address(&seeds.seeds(), &crate::ID);
        let faction_id = FactionId::MUD;

        // 1 for ix disc, 1 for
        let ix_data = [0, bump, faction_id as u8];
        let accounts = vec![
            AccountMeta::new(faction_account, false),
            AccountMeta::new(player_account.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ];
        let ix =
            solana_sdk::instruction::Instruction::new_with_bytes(crate::ID, &ix_data, accounts);
        let mut tx = solana_sdk::transaction::Transaction::new_with_payer(
            &[ix],
            Some(&player_account.pubkey()),
        );
        let rbh = client.get_latest_blockhash().await.unwrap();
        tx.sign(&[&player_account], rbh);
        let res = client
            .send_transaction_with_config(
                &tx,
                solana_client::rpc_config::RpcSendTransactionConfig {
                    skip_preflight: true,
                    preflight_commitment: None,
                    encoding: None,
                    max_retries: None,
                    min_context_slot: None,
                },
            )
            .await?;
        println!("Enlist txn: {res:?}");
        client.poll_for_signature(&res).await?;
        let tx = client
            .get_transaction_with_config(
                &res,
                RpcTransactionConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..Default::default()
                },
            )
            .await?;
        println!("Enlist txn res: {tx:#?}");
        let clock = client.get_block_time(tx.slot).await?;
        //
        // let expected_faction_account = PlayerFactionData {
        //     owner: player_account.pubkey(),
        //     enlisted_at_timestamp: clock,
        //     faction_id,
        //     bump,
        //     _padding: [0; 5],
        // };
        //
        // let faction_info = client.get_account(&faction_account).await?;
        // assert_eq!(faction_info.data[0..8], PlayerFactionData::DISCRIMINANT);
        // let new_faction: &PlayerFactionData = try_from_bytes(&faction_info.data[8..])?;
        // assert_eq!(expected_faction_account, *new_faction);

        Ok(())
    }
}
