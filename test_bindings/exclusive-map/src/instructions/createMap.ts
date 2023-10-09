// Generated Imports
import * as Web3 from '@solana/web3.js';
import * as DataSource from '@staratlas/data-source';
import * as Constants from '../constants';

// ---- BEGIN CUSTOM IMPORTS ----

// ----- END CUSTOM IMPORTS -----

export function createMap(
  program: Constants.ExclusiveMapIDLProgram,
  mapAccount: DataSource.AsyncSigner,
  authority: DataSource.AsyncSigner,
  remainingAccounts: Web3.AccountMeta[] = []
): DataSource.InstructionReturn {
  const asyncSignerReturn: DataSource.AsyncSigner[] = [mapAccount, authority];

  // ----- BEGIN CUSTOM LOGIC -----

  // ------ END CUSTOM LOGIC ------
  return async () => ({
    instruction: await program.methods
      .createMap()
      .accountsStrict({
        mapAccount: mapAccount.publicKey(),
        authority: authority.publicKey(),
        systemProgram: Web3.SystemProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .instruction(),
    signers: asyncSignerReturn,
  });
}
