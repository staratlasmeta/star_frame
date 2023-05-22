import { BN } from '@project-serum/anchor';
import { SystemProgram } from '@solana/web3.js';
import { AsyncSigner, InstructionReturn } from '@staratlas/data-source';
import { CustomClockIDLProgram } from '../constants';

export function initCustomClock(
  program: CustomClockIDLProgram,
  clock: AsyncSigner,
  slot: BN,
  timestamp: BN
): InstructionReturn {
  return async (funder) => ({
    instruction: await program.methods
      .initClock(slot, timestamp)
      .accountsStrict({
        funder: funder.publicKey(),
        clock: clock.publicKey(),
        systemProgram: SystemProgram.programId,
      })
      .instruction(),
    signers: [funder, clock],
  });
}
