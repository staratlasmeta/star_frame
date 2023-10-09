import * as Anchor from '@project-serum/anchor';
import * as Web3 from '@solana/web3.js';
import * as DataSource from '@staratlas/data-source';
import * as Types from '.';

export type InsertItems = {
  key: Web3.PublicKey;
  value: Types.ListValueUnpacked;
};
export function insertItemsEquals(
  insertItems1: InsertItems,
  insertItems2: InsertItems
): boolean {
  return (
    insertItems1.key.equals(insertItems2.key) &&
    Types.listValueUnpackedEquals(insertItems1.value, insertItems2.value)
  );
}

export type InsertItemsList = { items: Types.InsertItems[] };
export function insertItemsListEquals(
  insertItemsList1: InsertItemsList,
  insertItemsList2: InsertItemsList
): boolean {
  return DataSource.arrayDeepEquals(
    insertItemsList1.items,
    insertItemsList2.items,
    (a, b) => Types.insertItemsEquals(a, b)
  );
}

export type ListValue = {
  pubkey: Web3.PublicKey;
  byte: number;
  long: Anchor.BN;
};
export function listValueEquals(
  listValue1: ListValue,
  listValue2: ListValue
): boolean {
  return (
    listValue1.pubkey.equals(listValue2.pubkey) &&
    listValue1.byte === listValue2.byte &&
    listValue1.long.eq(listValue2.long)
  );
}

export type ListValueUnpacked = {
  pubkey: Web3.PublicKey;
  byte: number;
  long: Anchor.BN;
};
export function listValueUnpackedEquals(
  listValueUnpacked1: ListValueUnpacked,
  listValueUnpacked2: ListValueUnpacked
): boolean {
  return (
    listValueUnpacked1.pubkey.equals(listValueUnpacked2.pubkey) &&
    listValueUnpacked1.byte === listValueUnpacked2.byte &&
    listValueUnpacked1.long.eq(listValueUnpacked2.long)
  );
}
