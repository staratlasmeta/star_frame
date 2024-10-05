pub mod macro_prelude {
    pub use crate::account_set::{
        AccountSet, CanInitAccount, CanSetSeeds, HasOwnerProgram, HasProgramAccount, HasSeeds,
        SignedAccount, SingleAccountSet, SingleAccountSetMetadata, WritableAccount,
    };
    pub use crate::instruction::{Instruction, InstructionDiscriminant};
    pub use crate::syscalls::{SyscallAccountCache, SyscallInvoke};
    pub use crate::unsize::{
        AsBytes, AsMutBytes, FromBytesReturn, RefBytes, RefBytesMut, RefDeref, RefDerefMut,
        RefResize, RefWrapper, RefWrapperMutExt, RefWrapperTypes, Resize, UnsizedInit, UnsizedType,
        Zeroed, {CombinedExt, CombinedRef, CombinedUnsized, RefWrapperT, RefWrapperU},
    };
}
