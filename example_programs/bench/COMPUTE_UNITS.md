# Compute Units

(Using anchor's bench to generate with star frame's bench.so)

## [Unreleased]

Solana version: 2.1.0

| Instruction                 | Compute Units | -                       |
| --------------------------- | ------------- | ----------------------- |
| accountInfo1                | 163           | 🟢 **-408 (71.45%)**    |
| accountInfo2                | 183           | 🟢 **-712 (79.55%)**    |
| accountInfo4                | 213           | 🟢 **-1,340 (86.28%)**  |
| accountInfo8                | 276           | 🟢 **-2,647 (90.56%)**  |
| accountEmptyInit1           | 2,150         | 🟢 **-2,933 (57.70%)**  |
| accountEmpty1               | 221           | 🟢 **-424 (65.74%)**    |
| accountEmptyInit2           | 3,839         | 🟢 **-5,462 (58.72%)**  |
| accountEmpty2               | 302           | 🟢 **-705 (70.01%)**    |
| accountEmptyInit4           | 7,224         | 🟢 **-10,540 (59.33%)** |
| accountEmpty4               | 449           | 🟢 **-1,275 (73.96%)**  |
| accountEmptyInit8           | 13,985        | 🟢 **-20,738 (59.72%)** |
| accountEmpty8               | 757           | 🟢 **-2,406 (76.07%)**  |
| accountSizedInit1           | 2,152         | 🟢 **-3,040 (58.55%)**  |
| accountSized1               | 223           | 🟢 **-470 (67.82%)**    |
| accountSizedInit2           | 3,841         | 🟢 **-5,648 (59.52%)**  |
| accountSized2               | 303           | 🟢 **-772 (71.81%)**    |
| accountSizedInit4           | 7,227         | 🟢 **-10,943 (60.23%)** |
| accountSized4               | 449           | 🟢 **-1,399 (75.70%)**  |
| accountSizedInit8           | 13,993        | 🟢 **-21,440 (60.51%)** |
| accountSized8               | 756           | 🟢 **-2,631 (77.68%)**  |
| accountUnsizedInit1         | 2,172         | 🟢 **-3,133 (59.06%)**  |
| accountUnsized1             | 221           | 🟢 **-525 (70.38%)**    |
| accountUnsizedInit2         | 3,880         | 🟢 **-5,879 (60.24%)**  |
| accountUnsized2             | 303           | 🟢 **-860 (73.95%)**    |
| accountUnsizedInit4         | 7,306         | 🟢 **-11,297 (60.73%)** |
| accountUnsized4             | 449           | 🟢 **-1,553 (77.57%)**  |
| accountUnsizedInit8         | 14,152        | 🟢 **-21,841 (60.68%)** |
| accountUnsized8             | 757           | 🟢 **-2,916 (79.39%)**  |
| boxedAccountEmptyInit1      | 2,167         | 🟢 **-3,008 (58.13%)**  |
| boxedAccountEmpty1          | 260           | 🟢 **-474 (64.58%)**    |
| boxedAccountEmptyInit2      | 3,874         | 🟢 **-5,540 (58.85%)**  |
| boxedAccountEmpty2          | 380           | 🟢 **-736 (65.95%)**    |
| boxedAccountEmptyInit4      | 7,294         | 🟢 **-10,624 (59.29%)** |
| boxedAccountEmpty4          | 608           | 🟢 **-1,264 (67.52%)**  |
| boxedAccountEmptyInit8      | 14,119        | 🟢 **-20,834 (59.61%)** |
| boxedAccountEmpty8          | 1,075         | 🟢 **-2,326 (68.39%)**  |
| boxedAccountSizedInit1      | 2,168         | 🟢 **-3,103 (58.87%)**  |
| boxedAccountSized1          | 259           | 🟢 **-524 (66.92%)**    |
| boxedAccountSizedInit2      | 3,877         | 🟢 **-5,706 (59.54%)**  |
| boxedAccountSized2          | 379           | 🟢 **-811 (68.15%)**    |
| boxedAccountSizedInit4      | 7,298         | 🟢 **-10,932 (59.97%)** |
| boxedAccountSized4          | 609           | 🟢 **-1,387 (69.49%)**  |
| boxedAccountSizedInit8      | 14,126        | 🟢 **-21,427 (60.27%)** |
| boxedAccountSized8          | 1,076         | 🟢 **-2,552 (70.34%)**  |
| boxedAccountUnsizedInit1    | 2,188         | 🟢 **-3,183 (59.26%)**  |
| boxedAccountUnsized1        | 259           | 🟢 **-577 (69.02%)**    |
| boxedAccountUnsizedInit2    | 3,916         | 🟢 **-5,843 (59.87%)**  |
| boxedAccountUnsized2        | 380           | 🟢 **-890 (70.08%)**    |
| boxedAccountUnsizedInit4    | 7,378         | 🟢 **-11,180 (60.24%)** |
| boxedAccountUnsized4        | 608           | 🟢 **-1,524 (71.48%)**  |
| boxedAccountUnsizedInit8    | 14,286        | 🟢 **-21,899 (60.52%)** |
| boxedAccountUnsized8        | 1,075         | 🟢 **-2,806 (72.30%)**  |
| boxedInterfaceAccountMint1  | 312           | 🟢 **-1,039 (76.91%)**  |
| boxedInterfaceAccountMint2  | 481           | 🟢 **-1,642 (77.34%)**  |
| boxedInterfaceAccountMint4  | 802           | 🟢 **-2,854 (78.06%)**  |
| boxedInterfaceAccountMint8  | 1,451         | 🟢 **-5,287 (78.47%)**  |
| boxedInterfaceAccountToken1 | 311           | 🟢 **-1,700 (84.54%)**  |
| boxedInterfaceAccountToken2 | 478           | 🟢 **-2,953 (86.07%)**  |
| boxedInterfaceAccountToken4 | 798           | 🟢 **-5,462 (87.25%)**  |
| boxedInterfaceAccountToken8 | 1,443         | 🟢 **-10,491 (87.91%)** |
| interfaceAccountMint1       | 278           | 🟢 **-1,198 (81.17%)**  |
| interfaceAccountMint2       | 407           | 🟢 **-2,082 (83.65%)**  |
| interfaceAccountMint4       | 655           | 🟢 **-3,856 (85.48%)**  |
| interfaceAccountMint8       | 1,151         | 🟢 **-7,399 (86.54%)**  |
| interfaceAccountToken1      | 276           | 🟢 **-1,835 (86.93%)**  |
| interfaceAccountToken2      | 405           | 🟢 **-3,324 (89.14%)**  |
| interfaceAccountToken4      | 650           | 🟢 **-6,305 (90.65%)**  |
| program1                    | 173           | 🟢 **-606 (77.79%)**    |
| program2                    | 199           | 🟢 **-721 (78.37%)**    |
| program4                    | 240           | 🟢 **-953 (79.88%)**    |
| program8                    | 317           | 🟢 **-1,427 (81.82%)**  |
| signer1                     | 167           | 🟢 **-607 (78.42%)**    |
| signer2                     | 191           | 🟢 **-873 (82.05%)**    |
| signer4                     | 231           | 🟢 **-1,406 (85.89%)**  |
| signer8                     | 306           | 🟢 **-2,482 (89.02%)**  |
| systemAccount1              | 181           | 🟢 **-615 (77.26%)**    |
| systemAccount2              | 220           | 🟢 **-876 (79.93%)**    |
| systemAccount4              | 284           | 🟢 **-1,405 (83.19%)**  |
| systemAccount8              | 417           | 🟢 **-2,463 (85.52%)**  |
| uncheckedAccount1           | 162           | 🟢 **-621 (79.31%)**    |
| uncheckedAccount2           | 184           | 🟢 **-872 (82.58%)**    |
| uncheckedAccount4           | 215           | 🟢 **-1,379 (86.51%)**  |
| uncheckedAccount8           | 277           | 🟢 **-2,402 (89.66%)**  |
