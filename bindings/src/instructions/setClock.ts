import { BN } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { InstructionReturn } from '@staratlas/data-source';
import { CustomClockIDLProgram } from '../constants';

export function setClock(
  program: CustomClockIDLProgram,
  clock: PublicKey,
  slot: BN,
  timestamp: BN
): InstructionReturn {
  return async (funder) => ({
    instruction: await program.methods
      .setClock(slot, timestamp)
      .accountsStrict({
        clock,
      })
      .instruction(),
    signers: [funder],
  });
}
