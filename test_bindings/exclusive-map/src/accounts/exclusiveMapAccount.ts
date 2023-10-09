import * as Web3 from '@solana/web3.js';
import * as DataSource from '@staratlas/data-source';
import * as Constants from '../constants';
import * as Types from '../types';

export type ExclusiveMapAccountAccount = {
  version: number;
  authority: Web3.PublicKey;
  items: Types.InsertItems[];
};

export function exclusiveMapAccountEquals(
  exclusiveMapAccount1: ExclusiveMapAccountAccount,
  exclusiveMapAccount2: ExclusiveMapAccountAccount
): boolean {
  return (
    exclusiveMapAccount1.version === exclusiveMapAccount2.version &&
    exclusiveMapAccount1.authority.equals(exclusiveMapAccount2.authority) &&
    DataSource.arrayDeepEquals(
      exclusiveMapAccount1.items,
      exclusiveMapAccount2.items,
      (a, b) => Types.insertItemsEquals(a, b)
    )
  );
}

@DataSource.staticImplements<
  DataSource.AccountStatic<ExclusiveMapAccount, Constants.ExclusiveMapIDL>
>()
export class ExclusiveMapAccount implements DataSource.Account {
  static readonly ACCOUNT_NAME: NonNullable<
    Constants.ExclusiveMapIDL['accounts']
  >[number]['name'] = 'exclusiveMapAccount';

  static readonly MIN_DATA_SIZE: number =
    8 + // discriminator
    37;

  constructor(
    private _data: ExclusiveMapAccountAccount,
    private _key: Web3.PublicKey
  ) {}

  get data(): Readonly<ExclusiveMapAccountAccount> {
    return this._data;
  }

  get key(): Web3.PublicKey {
    return this._key;
  }

  static decodeData(
    account: Web3.KeyedAccountInfo,
    program: Constants.ExclusiveMapIDLProgram
  ): DataSource.DecodedAccountData<ExclusiveMapAccount> {
    return DataSource.decodeAccount(account, program, ExclusiveMapAccount);
  }
}
