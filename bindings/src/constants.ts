import { Coder, Program, Provider } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import {
  AnchorTypes,
  ExtractArrayType,
  ListenProgram,
  ListenProgramStatic,
  ProgramMethods,
  staticImplements,
} from '@staratlas/data-source';
import { CustomClock } from './customClock';
import {
  IDL as CUSTOM_CLOCK_IDL,
  CustomClock as CustomClockIDL,
} from './idl/custom_clock';

export { IDL as CUSTOM_CLOCK_IDL } from './idl/custom_clock';
export type { CustomClock as CustomClockIDL } from './idl/custom_clock';

export type CustomClockAccountsArray = ExtractArrayType<
  CustomClockIDL['accounts']
>['name'];

export type CustomClockIDLProgram = ProgramMethods<CustomClockIDL>;
export type CustomClockCoder = Coder<CustomClockAccountsArray, never>;

export type CustomClockTypes = AnchorTypes<CustomClockIDL>;

// Accounts
export type CustomClockIDLAccounts = CustomClockTypes['Accounts'];
export type CustomClockAccount = CustomClockIDLAccounts['customClock'];

export type CustomClockAccounts = {
  custom_clock: CustomClock;
};

@staticImplements<
  ListenProgramStatic<CustomClockProgram, CustomClockAccounts, CustomClockIDL>
>()
export class CustomClockProgram extends ListenProgram<
  CustomClockAccounts,
  CustomClockIDL
> {
  constructor(program: CustomClockIDLProgram) {
    super(program, {
      custom_clock: CustomClock,
    });
  }

  static buildProgram(
    programId: PublicKey,
    provider?: Provider,
    coder?: Coder
  ): CustomClockIDLProgram {
    return new Program(CUSTOM_CLOCK_IDL, programId, provider, coder);
  }
}
