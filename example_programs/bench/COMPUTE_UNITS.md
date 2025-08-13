# Compute Units

(Using anchor's bench to generate with star frame's bench.so)

## [Unreleased]

Solana version: 2.1.0

| Instruction                 | Compute Units | -                       |
| --------------------------- | ------------- | ----------------------- |
| accountInfo1                | 291           | 🟢 **-280 (49.04%)**    |
| accountInfo2                | 327           | 🟢 **-568 (63.46%)**    |
| accountInfo4                | 415           | 🟢 **-1,138 (73.28%)**  |
| accountInfo8                | 570           | 🟢 **-2,353 (80.50%)**  |
| accountEmptyInit1           | 2,770         | 🟢 **-2,313 (45.50%)**  |
| accountEmpty1               | 375           | 🟢 **-270 (41.86%)**    |
| accountEmptyInit2           | 4,832         | 🟢 **-4,469 (48.05%)**  |
| accountEmpty2               | 511           | 🟢 **-496 (49.26%)**    |
| accountEmptyInit4           | 8,955         | 🟢 **-8,809 (49.59%)**  |
| accountEmpty4               | 762           | 🟢 **-962 (55.80%)**    |
| accountEmptyInit8           | 17,247        | 🟢 **-17,476 (50.33%)** |
| accountEmpty8               | 1,285         | 🟢 **-1,878 (59.37%)**  |
| accountSizedInit1           | 2,774         | 🟢 **-2,418 (46.57%)**  |
| accountSized1               | 377           | 🟢 **-316 (45.60%)**    |
| accountSizedInit2           | 4,838         | 🟢 **-4,651 (49.01%)**  |
| accountSized2               | 512           | 🟢 **-563 (52.37%)**    |
| accountSizedInit4           | 8,966         | 🟢 **-9,204 (50.65%)**  |
| accountSized4               | 762           | 🟢 **-1,086 (58.77%)**  |
| accountSizedInit8           | 17,271        | 🟢 **-18,162 (51.26%)** |
| accountSized8               | 1,284         | 🟢 **-2,103 (62.09%)**  |
| accountUnsizedInit1         | 2,792         | 🟢 **-2,513 (47.37%)**  |
| accountUnsized1             | 375           | 🟢 **-371 (49.73%)**    |
| accountUnsizedInit2         | 4,873         | 🟢 **-4,886 (50.07%)**  |
| accountUnsized2             | 512           | 🟢 **-651 (55.98%)**    |
| accountUnsizedInit4         | 9,037         | 🟢 **-9,566 (51.42%)**  |
| accountUnsized4             | 762           | 🟢 **-1,240 (61.94%)**  |
| accountUnsizedInit8         | 17,414        | 🟢 **-18,579 (51.62%)** |
| accountUnsized8             | 1,285         | 🟢 **-2,388 (65.01%)**  |
| boxedAccountEmptyInit1      | 2,801         | 🟢 **-2,374 (45.87%)**  |
| boxedAccountEmpty1          | 392           | 🟢 **-342 (46.59%)**    |
| boxedAccountEmptyInit2      | 4,892         | 🟢 **-4,522 (48.03%)**  |
| boxedAccountEmpty2          | 540           | 🟢 **-576 (51.61%)**    |
| boxedAccountEmptyInit4      | 9,075         | 🟢 **-8,843 (49.35%)**  |
| boxedAccountEmpty4          | 829           | 🟢 **-1,043 (55.72%)**  |
| boxedAccountEmptyInit8      | 17,443        | 🟢 **-17,510 (50.10%)** |
| boxedAccountEmpty8          | 1,424         | 🟢 **-1,977 (58.13%)**  |
| boxedAccountSizedInit1      | 2,804         | 🟢 **-2,467 (46.80%)**  |
| boxedAccountSized1          | 391           | 🟢 **-392 (50.06%)**    |
| boxedAccountSizedInit2      | 4,899         | 🟢 **-4,684 (48.88%)**  |
| boxedAccountSized2          | 539           | 🟢 **-651 (54.71%)**    |
| boxedAccountSizedInit4      | 9,087         | 🟢 **-9,143 (50.15%)**  |
| boxedAccountSized4          | 830           | 🟢 **-1,166 (58.42%)**  |
| boxedAccountSizedInit8      | 17,466        | 🟢 **-18,087 (50.87%)** |
| boxedAccountSized8          | 1,425         | 🟢 **-2,203 (60.72%)**  |
| boxedAccountUnsizedInit1    | 2,822         | 🟢 **-2,549 (47.46%)**  |
| boxedAccountUnsized1        | 391           | 🟢 **-445 (53.23%)**    |
| boxedAccountUnsizedInit2    | 4,934         | 🟢 **-4,825 (49.44%)**  |
| boxedAccountUnsized2        | 540           | 🟢 **-730 (57.48%)**    |
| boxedAccountUnsizedInit4    | 9,159         | 🟢 **-9,399 (50.65%)**  |
| boxedAccountUnsized4        | 829           | 🟢 **-1,303 (61.12%)**  |
| boxedAccountUnsizedInit8    | 17,610        | 🟢 **-18,575 (51.33%)** |
| boxedAccountUnsized8        | 1,424         | 🟢 **-2,457 (63.31%)**  |
| boxedInterfaceAccountMint1  | 431           | 🟢 **-920 (68.10%)**    |
| boxedInterfaceAccountMint2  | 620           | 🟢 **-1,503 (70.80%)**  |
| boxedInterfaceAccountMint4  | 987           | 🟢 **-2,669 (73.00%)**  |
| boxedInterfaceAccountMint8  | 1,737         | 🟢 **-5,001 (74.22%)**  |
| boxedInterfaceAccountToken1 | 430           | 🟢 **-1,581 (78.62%)**  |
| boxedInterfaceAccountToken2 | 617           | 🟢 **-2,814 (82.02%)**  |
| boxedInterfaceAccountToken4 | 983           | 🟢 **-5,277 (84.30%)**  |
| boxedInterfaceAccountToken8 | 1,729         | 🟢 **-10,205 (85.51%)** |
| interfaceAccountMint1       | 416           | 🟢 **-1,060 (71.82%)**  |
| interfaceAccountMint2       | 589           | 🟢 **-1,900 (76.34%)**  |
| interfaceAccountMint4       | 919           | 🟢 **-3,592 (79.63%)**  |
| interfaceAccountMint8       | 1,598         | 🟢 **-6,952 (81.31%)**  |
| interfaceAccountToken1      | 414           | 🟢 **-1,697 (80.39%)**  |
| interfaceAccountToken2      | 587           | 🟢 **-3,142 (84.26%)**  |
| interfaceAccountToken4      | 914           | 🟢 **-6,041 (86.86%)**  |
| program1                    | 405           | 🟢 **-374 (48.01%)**    |
| program2                    | 557           | 🟢 **-363 (39.46%)**    |
| program4                    | 831           | 🟢 **-362 (30.34%)**    |
| program8                    | 1,425         | 🟢 **-319 (18.29%)**    |
| signer1                     | 301           | 🟢 **-473 (61.11%)**    |
| signer2                     | 356           | 🟢 **-708 (66.54%)**    |
| signer4                     | 443           | 🟢 **-1,194 (72.94%)**  |
| signer8                     | 663           | 🟢 **-2,125 (76.22%)**  |
| systemAccount1              | 334           | 🟢 **-462 (58.04%)**    |
| systemAccount2              | 434           | 🟢 **-662 (60.40%)**    |
| systemAccount4              | 597           | 🟢 **-1,092 (64.65%)**  |
| systemAccount8              | 976           | 🟢 **-1,904 (66.11%)**  |
| uncheckedAccount1           | 290           | 🟢 **-493 (62.96%)**    |
| uncheckedAccount2           | 328           | 🟢 **-728 (68.94%)**    |
| uncheckedAccount4           | 417           | 🟢 **-1,177 (73.84%)**  |
| uncheckedAccount8           | 571           | 🟢 **-2,108 (78.69%)**  |
