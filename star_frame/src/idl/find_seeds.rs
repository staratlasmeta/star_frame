use bytemuck::NoUninit;
use star_frame_idl::seeds::IdlFindSeed;

pub trait FindIdlSeeds {
    /// Returns the idl of this find seeds.
    fn find_seeds(&self) -> crate::IdlResult<Vec<IdlFindSeed>>;
}

impl FindIdlSeeds for Vec<IdlFindSeed> {
    fn find_seeds(&self) -> crate::IdlResult<Vec<IdlFindSeed>> {
        Ok(self.clone())
    }
}
impl FindIdlSeeds for &[IdlFindSeed] {
    fn find_seeds(&self) -> crate::IdlResult<Vec<IdlFindSeed>> {
        Ok(self.to_vec())
    }
}

impl<const N: usize> FindIdlSeeds for [IdlFindSeed; N] {
    fn find_seeds(&self) -> crate::IdlResult<Vec<IdlFindSeed>> {
        Ok(self.to_vec())
    }
}

#[must_use]
pub fn seed_const<T: NoUninit>(seed: T) -> FindSeed<T> {
    FindSeed::Const(seed)
}
/// Creates a seed that references an account path. For nested account sets, the fields should be split with a space. If
/// you want to specify that a path is from the root and not nested (even if it's nested in another account set), prefix
/// the path with a colon.
#[must_use]
pub fn seed_path<T: NoUninit>(path: &str) -> FindSeed<T> {
    FindSeed::Path(path.to_string())
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FindSeed<T: NoUninit> {
    Path(String),
    Const(T),
}

impl<T: NoUninit> From<FindSeed<T>> for IdlFindSeed {
    fn from(seed: FindSeed<T>) -> Self {
        match seed {
            FindSeed::Path(path) => IdlFindSeed::AccountPath(path),
            FindSeed::Const(constant) => {
                IdlFindSeed::Const(bytemuck::bytes_of::<T>(&constant).to_vec())
            }
        }
    }
}

impl<T: NoUninit> From<&FindSeed<T>> for IdlFindSeed {
    fn from(seed: &FindSeed<T>) -> Self {
        match seed {
            FindSeed::Path(path) => IdlFindSeed::AccountPath(path.clone()),
            FindSeed::Const(constant) => {
                IdlFindSeed::Const(bytemuck::bytes_of::<T>(constant).to_vec())
            }
        }
    }
}
