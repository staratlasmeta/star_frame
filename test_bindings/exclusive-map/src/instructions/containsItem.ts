// Generated Imports
import * as Web3 from '@solana/web3.js';
import * as DataSource from '@staratlas/data-source';
import * as Constants from '../constants';

// ---- BEGIN CUSTOM IMPORTS ----

// ----- END CUSTOM IMPORTS -----

export function containsItem(
  program: Constants.ExclusiveMapIDLProgram,
  mapAccount: Web3.PublicKey,
  authority: DataSource.AsyncSigner,
  keys: Web3.PublicKey[],
  remainingAccounts: Web3.AccountMeta[] = []
): DataSource.InstructionReturn {
  const asyncSignerReturn: DataSource.AsyncSigner[] = [authority];

  // ----- BEGIN CUSTOM LOGIC -----

  // ------ END CUSTOM LOGIC ------
  return async () => ({
    instruction: await program.methods
      .containsItem(keys)
      .accountsStrict({
        mapAccount,
        authority: authority.publicKey(),
        systemProgram: Web3.SystemProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .instruction(),
    signers: asyncSignerReturn,
  });
}
