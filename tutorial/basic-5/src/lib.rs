//! Basic-5: State Machines, Energy Systems, and Complex Game Logic
//!
//! This example demonstrates:
//! - State machine implementation for game entities
//! - Energy/resource management systems
//! - Complex validation logic with multiple conditions
//! - Time-based cooldowns and restrictions
//! - Saturating arithmetic for safe value updates
//!
//! Key concepts:
//! - Enum-based state machines with u8 storage
//! - Resource consumption and regeneration
//! - Complex state transition rules
//! - Tracking multiple game metrics
//! - Time-based game mechanics

use star_frame::{anyhow::ensure, prelude::*};

#[derive(StarFrameProgram)]
#[program(
    instruction_set = RobotInstructionSet,
    id = "B5sic55555555555555555555555555555555555555"
)]
pub struct RobotProgram;

/// Instruction set for controlling a robot with various actions
///
/// KEY PATTERN: Game-style instruction design
/// - Each instruction represents a distinct action
/// - Actions have different costs and effects
/// - State transitions are controlled through business logic
#[derive(InstructionSet)]
pub enum RobotInstructionSet {
    Initialize(Initialize),
    Walk(Walk),
    Run(Run),
    Jump(Jump),
    Rest(Rest),
}

/// State machine for robot behavior
///
/// DESIGN PATTERN: Enum with explicit discriminants
/// - Uses repr(u8) for efficient on-chain storage
/// - Explicit values ensure consistent serialization
/// - Default fallback in From implementation for safety
///
/// This is more efficient than storing strings and safer
/// than raw u8 values without type safety
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RobotState {
    Idle = 0,
    Walking = 1,
    Running = 2,
    Jumping = 3,
    Resting = 4,
}

/// Safe conversion from stored u8 to typed enum
///
/// KEY SAFETY: Always provide a fallback
/// - Protects against corrupted data
/// - Ensures program never panics on invalid state
/// - Uses Idle as safe default state
impl From<u8> for RobotState {
    fn from(value: u8) -> Self {
        match value {
            0 => RobotState::Idle,
            1 => RobotState::Walking,
            2 => RobotState::Running,
            3 => RobotState::Jumping,
            4 => RobotState::Resting,
            _ => RobotState::Idle,
        }
    }
}

/// Robot game entity with full state tracking
///
/// ARCHITECTURE: Complete game entity design
/// - State stored as u8 for efficiency
/// - Multiple resource counters (energy, distance, jumps)
/// - Time tracking for cooldowns
/// - Owner-based access control
///
/// This demonstrates a realistic on-chain game entity with
/// multiple interconnected systems
#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = RobotSeeds)]
#[repr(C, packed)]
pub struct RobotAccount {
    /// Owner who controls this robot
    pub owner: Pubkey,
    /// Current state (stored as u8, converted to enum)
    pub state: u8,
    /// Current energy level (0-100)
    pub energy: u64,
    /// Total distance traveled across all actions
    pub distance_traveled: u64,
    /// Number of jumps performed
    pub jumps_made: u32,
    /// Unix timestamp of last action (for cooldowns)
    pub last_action_time: i64,
}

/// PDA seeds for deterministic robot addresses
#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"robot")]
pub struct RobotSeeds {
    pub owner: Pubkey,
}

/// Game logic implementation for the robot
///
/// PATTERN: Encapsulated game logic
/// - Constants define game balance
/// - Helper methods provide safe state manipulation
/// - Validation methods enforce game rules
/// - Saturating arithmetic prevents overflows
///
/// This keeps all game logic in one place, making it
/// easier to balance and modify game mechanics
impl RobotAccount {
    /// Game balance constants
    const MAX_ENERGY: u64 = 100;
    const WALK_ENERGY_COST: u64 = 5;
    const RUN_ENERGY_COST: u64 = 10;
    const JUMP_ENERGY_COST: u64 = 20;
    const REST_ENERGY_GAIN: u64 = 25;

    /// Safe state getter with automatic conversion
    fn get_state(&self) -> RobotState {
        RobotState::from(self.state)
    }

    /// Safe state setter with automatic conversion
    fn set_state(&mut self, state: RobotState) {
        self.state = state as u8;
    }

    /// Validates if robot has enough energy for action
    ///
    /// KEY PATTERN: Descriptive error messages
    /// - Shows both required and available amounts
    /// - Helps debugging and user experience
    fn can_perform_action(&self, required_energy: u64) -> Result<()> {
        // Copy value to avoid unaligned reference in packed struct
        let current_energy = self.energy;
        ensure!(
            current_energy >= required_energy,
            "Insufficient energy: {} required, {} available",
            required_energy,
            current_energy
        );
        Ok(())
    }

    /// Safely consumes energy using saturating subtraction
    ///
    /// SAFETY: saturating_sub prevents underflow
    /// - Never goes below 0
    /// - No panic possible
    fn consume_energy(&mut self, amount: u64) {
        self.energy = self.energy.saturating_sub(amount);
    }

    /// Safely gains energy with maximum cap
    ///
    /// PATTERN: Chained saturating operations
    /// - saturating_add prevents overflow
    /// - min() enforces maximum cap
    /// - Common pattern for resource systems
    fn gain_energy(&mut self, amount: u64) {
        self.energy = self.energy.saturating_add(amount).min(Self::MAX_ENERGY);
    }
}

/// Owner validation for robot control
///
/// SECURITY: Simple but effective access control
/// - Only the owner can control their robot
/// - Prevents unauthorized actions
/// - Applied automatically via ValidatedAccount
impl AccountValidate<&Pubkey> for RobotAccount {
    fn validate_account(self_ref: &Self::Ref<'_>, owner: &Pubkey) -> Result<()> {
        ensure!(
            owner == &self_ref.owner,
            "Unauthorized: only owner can control the robot"
        );
        Ok(())
    }
}

/// Initialize instruction - Creates a new robot
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize;

/// Accounts for robot initialization
///
/// PATTERN: PDA initialization with Clock sysvar
/// - Owner pays for account creation (funder)
/// - Seeds derive from owner for uniqueness
/// - Clock provides initial timestamp
/// - System program handles account creation
#[derive(AccountSet)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub owner: Signer<Mut<SystemAccount>>,
    #[validate(arg = (
        Create(()),
        Seeds(RobotSeeds { owner: *self.owner.pubkey() }),
    ))]
    pub robot: Init<Seeded<Account<RobotAccount>>>,
    pub system_program: Program<System>,
}

/// Initialize a new robot at full energy
///
/// INITIALIZATION PATTERN:
/// - Starts at Idle state
/// - Full energy (ready to play)
/// - Zero stats (fresh start)
/// - Records creation time
///
/// This creates a ready-to-play game entity
impl StarFrameInstruction for Initialize {
    type ReturnType = ();
    type Accounts<'b, 'c> = InitializeAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        _run_arg: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        **accounts.robot.data_mut()? = RobotAccount {
            owner: *accounts.owner.pubkey(),
            state: RobotState::Idle as u8,
            energy: RobotAccount::MAX_ENERGY,
            distance_traveled: 0,
            jumps_made: 0,
            last_action_time: ctx.get_clock()?.unix_timestamp,
        };
        Ok(())
    }
}

/// Walk instruction with distance parameter
///
/// PATTERN: Parameterized game actions
/// - Takes runtime argument for distance
/// - Validates against current state
/// - Consumes resources
/// - Updates statistics
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Walk {
    #[ix_args(&run)]
    pub distance: u64,
}

/// Accounts for walk action
#[derive(AccountSet)]
pub struct WalkAccounts {
    pub owner: Signer,
    #[validate(arg = self.owner.pubkey())]
    pub robot: Mut<ValidatedAccount<RobotAccount>>,
}

/// Walk action implementation
///
/// GAME LOGIC PATTERN:
/// 1. Check energy requirements
/// 2. Validate current state
/// 3. Perform state transition
/// 4. Consume resources
/// 5. Update statistics
/// 6. Return to idle state
///
/// This pattern ensures consistent game mechanics
/// and prevents invalid state transitions
impl StarFrameInstruction for Walk {
    type ReturnType = ();
    type Accounts<'b, 'c> = WalkAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        distance: &u64,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut robot = accounts.robot.data_mut()?;

        // Step 1: Energy check
        robot.can_perform_action(RobotAccount::WALK_ENERGY_COST)?;

        // Step 2: State validation
        ensure!(
            robot.get_state() == RobotState::Idle || robot.get_state() == RobotState::Resting,
            "Robot must be idle or resting to walk"
        );

        // Step 3-6: Execute action
        robot.set_state(RobotState::Walking);
        robot.consume_energy(RobotAccount::WALK_ENERGY_COST);
        robot.distance_traveled = robot.distance_traveled.saturating_add(*distance);
        robot.last_action_time = ctx.get_clock()?.unix_timestamp;

        // Return to idle
        robot.set_state(RobotState::Idle);

        Ok(())
    }
}

/// Run instruction - Faster movement at higher energy cost
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Run {
    #[ix_args(&run)]
    pub distance: u64,
}

#[derive(AccountSet)]
pub struct RunAccounts {
    pub owner: Signer,
    #[validate(arg = self.owner.pubkey())]
    pub robot: Mut<ValidatedAccount<RobotAccount>>,
}

/// Run action - Covers 2x distance but uses more energy
///
/// GAME BALANCE EXAMPLE:
/// - Higher energy cost than walking
/// - Greater distance multiplier (2x)
/// - Can transition from idle or walking
/// - Strategic choice: efficiency vs speed
///
/// This shows how different actions can have
/// different cost-benefit tradeoffs
impl StarFrameInstruction for Run {
    type ReturnType = ();
    type Accounts<'b, 'c> = RunAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        distance: &u64,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut robot = accounts.robot.data_mut()?;

        robot.can_perform_action(RobotAccount::RUN_ENERGY_COST)?;

        // KEY: Different state requirements than Walk
        // Can accelerate from walking or start from idle
        ensure!(
            robot.get_state() == RobotState::Idle || robot.get_state() == RobotState::Walking,
            "Robot must be idle or walking to run"
        );

        robot.set_state(RobotState::Running);
        robot.consume_energy(RobotAccount::RUN_ENERGY_COST);
        // NOTE: 2x distance multiplier for running
        robot.distance_traveled = robot.distance_traveled.saturating_add(distance * 2);
        robot.last_action_time = ctx.get_clock()?.unix_timestamp;

        robot.set_state(RobotState::Idle);

        Ok(())
    }
}

/// Jump instruction - High energy action with fixed distance
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Jump;

#[derive(AccountSet)]
pub struct JumpAccounts {
    pub owner: Signer,
    #[validate(arg = self.owner.pubkey())]
    pub robot: Mut<ValidatedAccount<RobotAccount>>,
}

/// Jump action - Fixed distance, high energy cost
///
/// DESIGN CHOICE: No parameters
/// - Some actions have fixed effects
/// - Simplifies game balance
/// - Tracks jump statistics separately
/// - Different validation (can't jump while jumping)
///
/// Shows how not all game actions need to be parameterized
impl StarFrameInstruction for Jump {
    type ReturnType = ();
    type Accounts<'b, 'c> = JumpAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        _run_arg: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut robot = accounts.robot.data_mut()?;

        robot.can_perform_action(RobotAccount::JUMP_ENERGY_COST)?;

        // UNIQUE VALIDATION: Prevents double-jumping
        // Different from walk/run state checks
        ensure!(
            robot.get_state() != RobotState::Jumping,
            "Robot is already jumping"
        );

        robot.set_state(RobotState::Jumping);
        robot.consume_energy(RobotAccount::JUMP_ENERGY_COST);

        // Update multiple statistics
        robot.jumps_made = robot.jumps_made.saturating_add(1);
        robot.distance_traveled = robot.distance_traveled.saturating_add(5); // Fixed 5 units
        robot.last_action_time = ctx.get_clock()?.unix_timestamp;

        robot.set_state(RobotState::Idle);

        Ok(())
    }
}

/// Rest instruction - Regenerate energy with cooldown
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Rest;

#[derive(AccountSet)]
pub struct RestAccounts {
    pub owner: Signer,
    #[validate(arg = self.owner.pubkey())]
    pub robot: Mut<ValidatedAccount<RobotAccount>>,
}

/// Rest action - Energy regeneration with time restriction
///
/// COOLDOWN PATTERN:
/// - Enforces time between actions
/// - Prevents rapid energy farming
/// - Uses Clock sysvar for time validation
/// - Common pattern for rate-limiting in games
///
/// This demonstrates how to implement cooldowns
/// and prevent action spamming in on-chain games
impl StarFrameInstruction for Rest {
    type ReturnType = ();
    type Accounts<'b, 'c> = RestAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        _run_arg: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut robot = accounts.robot.data_mut()?;

        // TIME-BASED VALIDATION
        // Calculate time since last action
        let clock = ctx.get_clock()?;
        let time_since_last_action = clock.unix_timestamp - robot.last_action_time;

        // Enforce cooldown period
        ensure!(
            time_since_last_action >= 5,
            "Robot must wait at least 5 seconds between actions"
        );

        // Energy regeneration (not consumption)
        robot.set_state(RobotState::Resting);
        robot.gain_energy(RobotAccount::REST_ENERGY_GAIN);
        robot.last_action_time = ctx.get_clock()?.unix_timestamp;

        robot.set_state(RobotState::Idle);

        Ok(())
    }
}

/// Test module for IDL generation
///
/// TESTING PATTERN: IDL generation tests
/// - Validates program structure
/// - Generates Interface Definition Language files
/// - Used by client SDKs and explorers
/// - Run with: cargo test --features idl
#[cfg(test)]
mod tests {
    use star_frame::prelude::*;
    
    /// Generate IDL for client integration
    ///
    /// KEY TOOL: IDL (Interface Definition Language)
    /// - Describes program interface
    /// - Used by TypeScript/Rust/Python clients
    /// - Essential for SDK generation
    /// - Similar to ABI in Ethereum
    #[cfg(feature = "idl")]
    #[test]
    fn generate_idl() -> Result<()> {
        use crate::StarFrameDeclaredProgram;
        use codama_nodes::{NodeTrait, ProgramNode};
        let idl = StarFrameDeclaredProgram::program_to_idl()?;
        let codama_idl: ProgramNode = idl.try_into()?;
        let idl_json = codama_idl.to_json()?;
        std::fs::write("idl.json", &idl_json)?;
        Ok(())
    }
}
