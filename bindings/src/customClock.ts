import { KeyedAccountInfo, PublicKey } from '@solana/web3.js';
import {
  AccountStatic,
  DecodedAccountData,
  decodeAccount,
  staticImplements,
} from '@staratlas/data-source';
import {
  CustomClockAccount,
  CustomClockIDL,
  CustomClockIDLProgram,
} from './constants';

export function customClockDataEquals(
  customClockData1: CustomClockAccount,
  customClockData2: CustomClockAccount
): boolean {
  return (
    customClockData1.version === customClockData2.version &&
    customClockData1.slot.eq(customClockData2.slot) &&
    customClockData1.timestamp.eq(customClockData2.timestamp)
  );
}

@staticImplements<AccountStatic<CustomClock, CustomClockIDL>>()
export class CustomClock {
  static readonly ACCOUNT_NAME: NonNullable<
    CustomClockIDL['accounts']
  >[number]['name'] = 'customClock';
  static readonly MIN_DATA_SIZE =
    8 + // discriminator
    1 + // version
    8 + // slot
    8; // timestamp

  constructor(private _data: CustomClockAccount, private _key: PublicKey) {}

  get data(): CustomClockAccount {
    return this._data;
  }

  get key(): PublicKey {
    return this._key;
  }

  static decodeData(
    account: KeyedAccountInfo,
    program: CustomClockIDLProgram
  ): DecodedAccountData<CustomClock> {
    return decodeAccount(account, program, CustomClock);
  }
}
