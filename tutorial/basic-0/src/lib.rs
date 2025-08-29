//! Basic-0: Minimal Star Frame Program
//!
//! This example shows the absolute minimum required to create a Star Frame program.
//! Key concepts:
//! - StarFrameProgram derive macro
//! - Empty instruction set for programs with no instructions yet
//! - Program ID declaration

use star_frame::prelude::*;

// The StarFrameProgram derive macro generates the program entrypoint
// and all necessary boilerplate code
#[derive(StarFrameProgram)]
#[program(
    // Empty tuple means no instructions (yet)
    instruction_set = (),
    // Program ID - this would be your deployed program address
    id = "Basic11111111111111111111111111111111111111"
)]
pub struct BasicProgram;

#[cfg(test)]
mod tests {
    use star_frame::prelude::*;
    
    #[cfg(feature = "idl")]
    #[test]
    fn generate_idl() -> Result<()> {
        use crate::StarFrameDeclaredProgram;
        use codama_nodes::{NodeTrait, ProgramNode};
        let idl = StarFrameDeclaredProgram::program_to_idl()?;
        let codama_idl: ProgramNode = idl.try_into()?;
        let idl_json = codama_idl.to_json()?;
        std::fs::write("idl.json", &idl_json)?;
        println!("{idl_json}");
        Ok(())
    }
}
