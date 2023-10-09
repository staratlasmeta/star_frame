import * as Anchor from '@project-serum/anchor';
import * as Web3 from '@solana/web3.js';
import * as DataSource from '@staratlas/data-source';

import {
  IDL as EXCLUSIVE_MAP_IDL,
  ExclusiveMap as ExclusiveMapIDL,
} from './idl/exclusive_map';

// Account Imports
import { ExclusiveMapAccount } from './accounts';

// Global Exports
export { EXCLUSIVE_MAP_IDL, ExclusiveMapIDL };
export const exclusiveMapErrorMap =
  DataSource.generateErrorMap(EXCLUSIVE_MAP_IDL);
export type ExclusiveMapIDLProgram = DataSource.ProgramMethods<ExclusiveMapIDL>;
export type ExclusiveMapTypes = DataSource.AnchorTypes<ExclusiveMapIDL>;
export type ExclusiveMapAccountsArray = DataSource.ExtractArrayType<
  ExclusiveMapIDL['accounts']
>['name'];
export type ExclusiveMapCoder = Anchor.Coder<
  ExclusiveMapAccountsArray,
  ExclusiveMapTypesArray
>;
export type ExclusiveMapTypesArray = DataSource.ExtractArrayType<
  ExclusiveMapIDL['types']
>['name'];

// Accounts Export
export type ExclusiveMapAccounts = { exclusiveMapAccount: ExclusiveMapAccount };

@DataSource.staticImplements<
  DataSource.ListenProgramStatic<
    ExclusiveMapProgram,
    ExclusiveMapAccounts,
    ExclusiveMapIDL
  >
>()
export class ExclusiveMapProgram extends DataSource.ListenProgram<
  ExclusiveMapAccounts,
  ExclusiveMapIDL
> {
  constructor(program: ExclusiveMapIDLProgram) {
    super(program, { exclusiveMapAccount: ExclusiveMapAccount });
  }

  static buildProgram(
    programId: Web3.PublicKey,
    provider?: Anchor.Provider,
    coder?: Anchor.Coder
  ): ExclusiveMapIDLProgram {
    return new Anchor.Program(EXCLUSIVE_MAP_IDL, programId, provider, coder);
  }
}
