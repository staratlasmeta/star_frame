// Generated Imports
import * as Web3 from '@solana/web3.js';
import * as DataSource from '@staratlas/data-source';
import * as Constants from '../constants';
import * as Types from '../types';

// ---- BEGIN CUSTOM IMPORTS ----

// ----- END CUSTOM IMPORTS -----

export function insertItems(
  program: Constants.ExclusiveMapIDLProgram,
  mapAccount: Web3.PublicKey,
  authority: DataSource.AsyncSigner,
  items: Types.InsertItemsList,
  remainingAccounts: Web3.AccountMeta[] = []
): DataSource.InstructionReturn {
  const asyncSignerReturn: DataSource.AsyncSigner[] = [authority];

  // ----- BEGIN CUSTOM LOGIC -----

  // ------ END CUSTOM LOGIC ------
  return async () => ({
    instruction: await program.methods
      .insertItems(items as never)
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
