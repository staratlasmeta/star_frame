import { AnchorProvider, BN, Wallet } from '@project-serum/anchor';
import {
  ComputeBudgetProgram,
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
} from '@solana/web3.js';
import {
  AsyncSigner,
  InstructionWithSigners,
  airdrop,
  buildSendAndCheck,
  formatExplorerLink,
  keypairToAsyncSigner,
  readFromRPCOrError,
} from '@staratlas/data-source';
// @ts-ignore
// import { default as authKey } from './map-auth-key.json';
// @ts-ignore
// import { default as mapKey } from './map-key.json';
import {
  ExclusiveMapAccount,
  ExclusiveMapProgram,
  InsertItems,
  InsertItemsList,
  containsItem,
  createMap,
  deleteItems,
  insertItems,
} from '../src';

describe('exclusive-map-ts', () => {
  const PROGRAM_ID = new PublicKey(
    'SoRt3CQHw4RuakqngB6X3ZgBV5nya3c2NECg3rS8wZS'
  );
  const connection = new Connection('http://localhost:8899', 'confirmed');
  const AUTH_KEYPAIR = Keypair.generate();
  const MAP_KEYPAIR = Keypair.generate();
  // const AUTH_KEYPAIR = Keypair.fromSecretKey(Uint8Array.from(authKey));
  // const MAP_KEYPAIR = Keypair.fromSecretKey(Uint8Array.from(mapKey));
  console.log('Map key: ', MAP_KEYPAIR.publicKey.toBase58());

  const provider = new AnchorProvider(connection, new Wallet(MAP_KEYPAIR), {
    commitment: 'confirmed',
  });
  const exclusiveMap = ExclusiveMapProgram.buildProgram(PROGRAM_ID, provider);

  const map_signer = keypairToAsyncSigner(MAP_KEYPAIR);
  const authority_signer = keypairToAsyncSigner(AUTH_KEYPAIR);

  const mapItems: InsertItems[] = [];

  beforeAll(async () => {
    await airdrop(connection, AUTH_KEYPAIR.publicKey, LAMPORTS_PER_SOL * 100);
  });

  it('should create a map', async () => {
    const mapIx = createMap(exclusiveMap, map_signer, authority_signer);
    const createMapIx = await buildSendAndCheck(
      mapIx,
      authority_signer,
      connection,
      { sendOptions: { skipPreflight: true } }
    );
    console.log(formatExplorerLink(createMapIx, connection));
  });

  it('should insert a bunch of items in the list', async () => {
    // const saveItems: InsertItems[] = [];
    const TRANSACTION_COUNT = 100;
    const ITEMS_PER_TRANSACTION = 10;
    await Promise.all(
      new Array(TRANSACTION_COUNT).fill(0).map(async () => {
        const itemsToInsert: InsertItems[] = [];
        for (let i = 0; i < ITEMS_PER_TRANSACTION; i++) {
          const itemToInsert: InsertItems = {
            // key: SystemProgram.programId,
            key: Keypair.generate().publicKey,
            value: {
              byte: 0,
              long: new BN(Math.random() * 10000),
              pubkey: Keypair.generate().publicKey,
            },
          };
          // saveItems.push(itemToInsert);
          mapItems.push(itemToInsert);
          itemsToInsert.push(itemToInsert);
        }
        const items: InsertItemsList = {
          items: itemsToInsert,
        };
        const insertIx = insertItems(
          exclusiveMap,
          map_signer.publicKey(),
          authority_signer,
          items
        );
        const computeBudgetIx = async (
          _signer: AsyncSigner
        ): Promise<InstructionWithSigners> => {
          return {
            instruction: ComputeBudgetProgram.setComputeUnitLimit({
              units: 65000,
            }),
            signers: [],
          };
        };
        const txn = await buildSendAndCheck(
          [computeBudgetIx, insertIx],
          // insertIx,
          authority_signer,
          connection,
          { sendOptions: { skipPreflight: true } }
        );
        console.log(formatExplorerLink(txn, connection));
      })
    );
    // sleep for 3 secs
    // await new Promise((resolve) => setTimeout(resolve, 3000));
    const account = await readFromRPCOrError(
      connection,
      exclusiveMap,
      MAP_KEYPAIR.publicKey,
      ExclusiveMapAccount,
      'processed'
    );
    const expectedLength = TRANSACTION_COUNT * ITEMS_PER_TRANSACTION;
    const mapStuff = account.data.items;
    console.log('Map stuff: ', mapStuff);
    expect(mapStuff.length).toEqual(expectedLength);
    // // read from the file
    // let itemsToWrite: InsertItems[] = [];
    // if (existsSync('items.json')) {
    //   const itemsFile = readFileSync('items.json');
    //   itemsToWrite = JSON.parse(itemsFile.toString());
    // }
    // itemsToWrite.push(...saveItems);
    // writeFileSync('items.json', JSON.stringify(itemsToWrite, null, 2));
  }, 1000000);

  it('should contain all the items', async () => {
    // let itemsToCheck: InsertItems[] = [];
    // if (existsSync('items.json')) {
    //   const itemsFile = readFileSync('items.json');
    //   itemsToCheck = JSON.parse(itemsFile.toString());
    // }
    const itemsToCheck = [...mapItems];
    const ixs = [];

    while (itemsToCheck.length > 0) {
      const items = itemsToCheck.splice(0, 20);
      const ix = containsItem(
        exclusiveMap,
        MAP_KEYPAIR.publicKey,
        authority_signer,
        items.map((item) => new PublicKey(item.key))
      );
      ixs.push(ix);
    }
    await Promise.all(
      ixs.map(async (ix) => {
        const txn = await buildSendAndCheck(ix, authority_signer, connection, {
          sendOptions: { skipPreflight: true },
        });
        console.log(formatExplorerLink(txn, connection));
      })
    );
  }, 100000);

  it('should delete all the items', async () => {
    // let itemsToDelete: InsertItems[] = [];
    // if (existsSync('items.json')) {
    //   const itemsFile = readFileSync('items.json');
    //   itemsToDelete = JSON.parse(itemsFile.toString());
    // }
    const itemsToDelete = [...mapItems];
    const ixs = [];

    while (itemsToDelete.length > 0) {
      const items = itemsToDelete.splice(0, 20);
      const ix = deleteItems(
        exclusiveMap,
        MAP_KEYPAIR.publicKey,
        authority_signer,
        items.map((item) => new PublicKey(item.key))
      );
      ixs.push(ix);
    }

    await Promise.all(
      ixs.map(async (ix) => {
        const txn = await buildSendAndCheck(ix, authority_signer, connection, {
          sendOptions: { skipPreflight: true },
        });
        console.log(formatExplorerLink(txn, connection));
      })
    );
    const account = await readFromRPCOrError(
      connection,
      exclusiveMap,
      MAP_KEYPAIR.publicKey,
      ExclusiveMapAccount
    );
    const mapStuff = account.data.items;
    expect(mapStuff.length).toEqual(0);
  }, 1000000);

  it.skip('should contain an item', async () => {
    // const key = new PublicKey('52UdKDutVsme2p5boErDDqc66y3oVBEwQd7dr1xCovEG');
    // const key = SystemProgram.programId;
    const key = Keypair.generate().publicKey;
    const ix = containsItem(
      exclusiveMap,
      MAP_KEYPAIR.publicKey,
      authority_signer,
      [key]
    );
    const txn = await buildSendAndCheck(ix, authority_signer, connection, {
      sendOptions: { skipPreflight: true },
    });
    console.log(formatExplorerLink(txn, connection));
  });
});
