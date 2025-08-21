use star_frame::prelude::*;

use instructions::{CancelOrders, Initialize, PlaceOrder};
mod instructions;
pub mod state;

#[derive(StarFrameProgram)]
#[program(
    instruction_set = MarketplaceInstructionSet,
    id = Pubkey::new_from_array([10; 32])
)]
pub struct Marketplace;

#[derive(InstructionSet)]
pub enum MarketplaceInstructionSet {
    Initialize(Initialize),
    PlaceOrder(PlaceOrder),
    CancelOrders(CancelOrders),
}

#[cfg(test)]
pub mod test_utils {
    use super::*;

    use mollusk_svm::Mollusk;
    use solana_account::Account as SolanaAccount;
    use star_frame::{data_types::PackedValue, solana_pubkey::Pubkey};
    use star_frame_spl::token::{state::MintAccount, Token};

    use crate::state::{Price, Quantity};

    pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
    pub const TOKEN_SUPPLY: u64 = 100_000_000_000;
    pub const TOKEN_DECIMALS: u8 = 0;

    pub fn new_price(v: u64) -> Price {
        Price::new(PackedValue(v))
    }

    pub fn new_quantity(v: u64) -> Quantity {
        Quantity::new(PackedValue(v))
    }

    pub fn new_mint_account(mint: KeyFor<MintAccount>) -> (Pubkey, SolanaAccount) {
        let acc = SolanaAccount {
            lamports: LAMPORTS_PER_SOL,
            data: bytemuck::bytes_of(&star_frame_spl::token::state::MintAccountData {
                mint_authority: star_frame_spl::pod::PodOption::none(),
                supply: TOKEN_SUPPLY,
                decimals: TOKEN_DECIMALS,
                is_initialized: true,
                freeze_authority: star_frame_spl::pod::PodOption::none(),
            })
            .to_vec(),
            owner: Token::ID,
            executable: false,
            rent_epoch: 0,
        };
        (*mint.pubkey(), acc)
    }

    pub fn token_account_data(owner: Pubkey, mint: KeyFor<MintAccount>, amount: u64) -> Vec<u8> {
        bytemuck::bytes_of(&star_frame_spl::token::state::TokenAccountData {
            mint,
            owner,
            amount,
            delegate: star_frame_spl::pod::PodOption::none(),
            state: star_frame_spl::token::state::AccountState::Initialized,
            is_native: star_frame_spl::pod::PodOption::none(),
            delegated_amount: 0,
            close_authority: star_frame_spl::pod::PodOption::none(),
        })
        .to_vec()
    }

    pub fn new_token_account(
        key: Pubkey,
        owner: Pubkey,
        mint: KeyFor<MintAccount>,
        amount: u64,
    ) -> (Pubkey, SolanaAccount) {
        let acc = SolanaAccount {
            lamports: LAMPORTS_PER_SOL,
            data: token_account_data(owner, mint, amount),
            owner: Token::ID,
            executable: false,
            rent_epoch: 0,
        };
        (key, acc)
    }

    pub fn new_mollusk() -> Mollusk {
        let mut mollusk = Mollusk::new(&crate::Marketplace::ID, "marketplace");
        mollusk_svm_programs_token::token::add_program(&mut mollusk);
        mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
        mollusk
    }
}

#[cfg(test)]
mod idl_test {
    use super::*;

    #[cfg(feature = "idl")]
    #[test]
    fn idl() {
        let idl: star_frame::star_frame_idl::ProgramNode =
            Marketplace::program_to_idl().unwrap().try_into().unwrap();
        let idl_json = star_frame::serde_json::to_string_pretty(&idl).unwrap();
        println!("{idl_json}",);
        std::fs::write("idl.json", &idl_json).unwrap();
    }
}
