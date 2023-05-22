/// Type safe pda seeding.
pub trait Seeds<'a, 'info: 'a, const N: usize> {
    /// The unique discriminator for the PDA.
    const DISCRIMINATOR: &'static [u8];
    /// The arg used to build the seeds.
    type SeedsArg;

    /// Converts this to a set of seeds including the DISCRIMINATOR and bump.
    /// This is not actually used because anchor doesn't allow it...
    fn signer_seeds(arg: &'a Self::SeedsArg) -> [&'a [u8]; N];
}
