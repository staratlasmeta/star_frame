use star_frame::prelude::*;

use instructions::Initialize;
mod instructions;
mod state;

#[derive(StarFrameProgram)]
#[program(
    instruction_set = MarketplaceInstructionSet,
    id = Pubkey::new_from_array([10; 32])
)]
pub struct Marketplace;

#[derive(InstructionSet)]
pub enum MarketplaceInstructionSet {
    Initialize(Initialize),
}

#[cfg(test)]
mod tests {
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
