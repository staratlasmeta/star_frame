export type ExclusiveMap = {
  version: '0.0.0';
  name: 'exclusive_map';
  instructions: [
    {
      name: 'containsItem';
      accounts: [
        {
          name: 'mapAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'keys';
          type: {
            vec: 'publicKey';
          };
        }
      ];
    },
    {
      name: 'createMap';
      docs: ['Initializes a new exclusive map account.'];
      accounts: [
        {
          name: 'mapAccount';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'authority';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [];
    },
    {
      name: 'deleteItems';
      accounts: [
        {
          name: 'mapAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'keys';
          type: {
            vec: 'publicKey';
          };
        }
      ];
    },
    {
      name: 'insertItems';
      docs: ['Initializes a new exclusive map account.'];
      accounts: [
        {
          name: 'mapAccount';
          isMut: true;
          isSigner: false;
        },
        {
          name: 'authority';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'systemProgram';
          isMut: false;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'items';
          type: {
            defined: 'InsertItemsList';
          };
        }
      ];
    }
  ];
  accounts: [
    {
      name: 'exclusiveMapAccount';
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'version';
            type: 'u8';
          },
          {
            name: 'authority';
            type: 'publicKey';
          },
          {
            name: 'items';
            type: {
              vec: {
                defined: 'InsertItems';
              };
            };
          }
        ];
      };
    }
  ];
  types: [
    {
      name: 'InsertItems';
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'key';
            type: 'publicKey';
          },
          {
            name: 'value';
            type: {
              defined: 'ListValueUnpacked';
            };
          }
        ];
      };
    },
    {
      name: 'InsertItemsList';
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'items';
            type: {
              vec: {
                defined: 'InsertItems';
              };
            };
          }
        ];
      };
    },
    {
      name: 'ListValue';
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'pubkey';
            type: 'publicKey';
          },
          {
            name: 'byte';
            type: 'u8';
          },
          {
            name: 'long';
            type: 'u64';
          }
        ];
      };
    },
    {
      name: 'ListValueUnpacked';
      docs: ['Unpacked version of [`ListValue`]'];
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'pubkey';
            type: 'publicKey';
          },
          {
            name: 'byte';
            type: 'u8';
          },
          {
            name: 'long';
            type: 'u64';
          }
        ];
      };
    }
  ];
};

export const IDL: ExclusiveMap = {
  version: '0.0.0',
  name: 'exclusive_map',
  instructions: [
    {
      name: 'containsItem',
      accounts: [
        {
          name: 'mapAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: true,
          isSigner: true,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'keys',
          type: {
            vec: 'publicKey',
          },
        },
      ],
    },
    {
      name: 'createMap',
      docs: ['Initializes a new exclusive map account.'],
      accounts: [
        {
          name: 'mapAccount',
          isMut: true,
          isSigner: true,
        },
        {
          name: 'authority',
          isMut: true,
          isSigner: true,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [],
    },
    {
      name: 'deleteItems',
      accounts: [
        {
          name: 'mapAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: true,
          isSigner: true,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'keys',
          type: {
            vec: 'publicKey',
          },
        },
      ],
    },
    {
      name: 'insertItems',
      docs: ['Initializes a new exclusive map account.'],
      accounts: [
        {
          name: 'mapAccount',
          isMut: true,
          isSigner: false,
        },
        {
          name: 'authority',
          isMut: true,
          isSigner: true,
        },
        {
          name: 'systemProgram',
          isMut: false,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'items',
          type: {
            defined: 'InsertItemsList',
          },
        },
      ],
    },
  ],
  accounts: [
    {
      name: 'exclusiveMapAccount',
      type: {
        kind: 'struct',
        fields: [
          {
            name: 'version',
            type: 'u8',
          },
          {
            name: 'authority',
            type: 'publicKey',
          },
          {
            name: 'items',
            type: {
              vec: {
                defined: 'InsertItems',
              },
            },
          },
        ],
      },
    },
  ],
  types: [
    {
      name: 'InsertItems',
      type: {
        kind: 'struct',
        fields: [
          {
            name: 'key',
            type: 'publicKey',
          },
          {
            name: 'value',
            type: {
              defined: 'ListValueUnpacked',
            },
          },
        ],
      },
    },
    {
      name: 'InsertItemsList',
      type: {
        kind: 'struct',
        fields: [
          {
            name: 'items',
            type: {
              vec: {
                defined: 'InsertItems',
              },
            },
          },
        ],
      },
    },
    {
      name: 'ListValue',
      type: {
        kind: 'struct',
        fields: [
          {
            name: 'pubkey',
            type: 'publicKey',
          },
          {
            name: 'byte',
            type: 'u8',
          },
          {
            name: 'long',
            type: 'u64',
          },
        ],
      },
    },
    {
      name: 'ListValueUnpacked',
      docs: ['Unpacked version of [`ListValue`]'],
      type: {
        kind: 'struct',
        fields: [
          {
            name: 'pubkey',
            type: 'publicKey',
          },
          {
            name: 'byte',
            type: 'u8',
          },
          {
            name: 'long',
            type: 'u64',
          },
        ],
      },
    },
  ],
};
