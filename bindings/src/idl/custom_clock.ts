export type CustomClock = {
  version: '0.1.0';
  name: 'custom_clock';
  instructions: [
    {
      name: 'initClock';
      accounts: [
        {
          name: 'funder';
          isMut: true;
          isSigner: true;
        },
        {
          name: 'clock';
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
          name: 'slot';
          type: 'u64';
        },
        {
          name: 'timestamp';
          type: 'i64';
        }
      ];
    },
    {
      name: 'setClock';
      accounts: [
        {
          name: 'clock';
          isMut: true;
          isSigner: false;
        }
      ];
      args: [
        {
          name: 'slot';
          type: 'u64';
        },
        {
          name: 'timestamp';
          type: 'i64';
        }
      ];
    }
  ];
  accounts: [
    {
      name: 'customClock';
      type: {
        kind: 'struct';
        fields: [
          {
            name: 'version';
            type: 'u8';
          },
          {
            name: 'slot';
            type: 'u64';
          },
          {
            name: 'timestamp';
            type: 'i64';
          }
        ];
      };
    }
  ];
};

export const IDL: CustomClock = {
  version: '0.1.0',
  name: 'custom_clock',
  instructions: [
    {
      name: 'initClock',
      accounts: [
        {
          name: 'funder',
          isMut: true,
          isSigner: true,
        },
        {
          name: 'clock',
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
          name: 'slot',
          type: 'u64',
        },
        {
          name: 'timestamp',
          type: 'i64',
        },
      ],
    },
    {
      name: 'setClock',
      accounts: [
        {
          name: 'clock',
          isMut: true,
          isSigner: false,
        },
      ],
      args: [
        {
          name: 'slot',
          type: 'u64',
        },
        {
          name: 'timestamp',
          type: 'i64',
        },
      ],
    },
  ],
  accounts: [
    {
      name: 'customClock',
      type: {
        kind: 'struct',
        fields: [
          {
            name: 'version',
            type: 'u8',
          },
          {
            name: 'slot',
            type: 'u64',
          },
          {
            name: 'timestamp',
            type: 'i64',
          },
        ],
      },
    },
  ],
};
