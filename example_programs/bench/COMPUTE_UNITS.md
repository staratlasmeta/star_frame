# Compute Units

(Using anchor's bench to generate with star frame's bench.so)

## [Unreleased]

Solana version: 2.1.0

| Instruction                 | Compute Units | -                       |
| --------------------------- | ------------- | ----------------------- |
| accountInfo1                | 165           | 🟢 **-406 (71.10%)**    |
| accountInfo2                | 185           | 🟢 **-710 (79.33%)**    |
| accountInfo4                | 215           | 🟢 **-1,338 (86.16%)**  |
| accountInfo8                | 278           | 🟢 **-2,645 (90.49%)**  |
| accountEmptyInit1           | 1,989         | 🟢 **-3,094 (60.87%)**  |
| accountEmpty1               | 193           | 🟢 **-452 (70.08%)**    |
| accountEmptyInit2           | 3,592         | 🟢 **-5,709 (61.38%)**  |
| accountEmpty2               | 245           | 🟢 **-762 (75.67%)**    |
| accountEmptyInit4           | 6,805         | 🟢 **-10,959 (61.69%)** |
| accountEmpty4               | 336           | 🟢 **-1,388 (80.51%)**  |
| accountEmptyInit8           | 13,218        | 🟢 **-21,505 (61.93%)** |
| accountEmpty8               | 520           | 🟢 **-2,643 (83.56%)**  |
| accountSizedInit1           | 1,997         | 🟢 **-3,195 (61.54%)**  |
| accountSized1               | 195           | 🟢 **-498 (71.86%)**    |
| accountSizedInit2           | 3,606         | 🟢 **-5,883 (62.00%)**  |
| accountSized2               | 246           | 🟢 **-829 (77.12%)**    |
| accountSizedInit4           | 6,832         | 🟢 **-11,338 (62.40%)** |
| accountSized4               | 336           | 🟢 **-1,512 (81.82%)**  |
| accountSizedInit8           | 13,274        | 🟢 **-22,159 (62.54%)** |
| accountSized8               | 519           | 🟢 **-2,868 (84.68%)**  |
| accountUnsizedInit1         | 1,997         | 🟢 **-3,308 (62.36%)**  |
| accountUnsized1             | 193           | 🟢 **-553 (74.13%)**    |
| accountUnsizedInit2         | 3,605         | 🟢 **-6,154 (63.06%)**  |
| accountUnsized2             | 246           | 🟢 **-917 (78.85%)**    |
| accountUnsizedInit4         | 6,831         | 🟢 **-11,772 (63.28%)** |
| accountUnsized4             | 336           | 🟢 **-1,666 (83.22%)**  |
| accountUnsizedInit8         | 13,273        | 🟢 **-22,720 (63.12%)** |
| accountUnsized8             | 520           | 🟢 **-3,153 (85.84%)**  |
| boxedAccountEmptyInit1      | 2,009         | 🟢 **-3,166 (61.18%)**  |
| boxedAccountEmpty1          | 211           | 🟢 **-523 (71.25%)**    |
| boxedAccountEmptyInit2      | 3,631         | 🟢 **-5,783 (61.43%)**  |
| boxedAccountEmpty2          | 282           | 🟢 **-834 (74.73%)**    |
| boxedAccountEmptyInit4      | 6,881         | 🟢 **-11,037 (61.60%)** |
| boxedAccountEmpty4          | 408           | 🟢 **-1,464 (78.21%)**  |
| boxedAccountEmptyInit8      | 13,370        | 🟢 **-21,583 (61.75%)** |
| boxedAccountEmpty8          | 672           | 🟢 **-2,729 (80.24%)**  |
| boxedAccountSizedInit1      | 2,016         | 🟢 **-3,255 (61.75%)**  |
| boxedAccountSized1          | 210           | 🟢 **-573 (73.18%)**    |
| boxedAccountSizedInit2      | 3,646         | 🟢 **-5,937 (61.95%)**  |
| boxedAccountSized2          | 281           | 🟢 **-909 (76.39%)**    |
| boxedAccountSizedInit4      | 6,909         | 🟢 **-11,321 (62.10%)** |
| boxedAccountSized4          | 409           | 🟢 **-1,587 (79.51%)**  |
| boxedAccountSizedInit8      | 13,425        | 🟢 **-22,128 (62.24%)** |
| boxedAccountSized8          | 673           | 🟢 **-2,955 (81.45%)**  |
| boxedAccountUnsizedInit1    | 2,016         | 🟢 **-3,355 (62.47%)**  |
| boxedAccountUnsized1        | 210           | 🟢 **-626 (74.88%)**    |
| boxedAccountUnsizedInit2    | 3,645         | 🟢 **-6,114 (62.65%)**  |
| boxedAccountUnsized2        | 282           | 🟢 **-988 (77.80%)**    |
| boxedAccountUnsizedInit4    | 6,909         | 🟢 **-11,649 (62.77%)** |
| boxedAccountUnsized4        | 408           | 🟢 **-1,724 (80.86%)**  |
| boxedAccountUnsizedInit8    | 13,425        | 🟢 **-22,760 (62.90%)** |
| boxedAccountUnsized8        | 672           | 🟢 **-3,209 (82.68%)**  |
| boxedInterfaceAccountMint1  | 233           | 🟢 **-1,118 (82.75%)**  |
| boxedInterfaceAccountMint2  | 323           | 🟢 **-1,800 (84.79%)**  |
| boxedInterfaceAccountMint4  | 483           | 🟢 **-3,173 (86.79%)**  |
| boxedInterfaceAccountMint8  | 814           | 🟢 **-5,924 (87.92%)**  |
| boxedInterfaceAccountToken1 | 232           | 🟢 **-1,779 (88.46%)**  |
| boxedInterfaceAccountToken2 | 320           | 🟢 **-3,111 (90.67%)**  |
| boxedInterfaceAccountToken4 | 479           | 🟢 **-5,781 (92.35%)**  |
| boxedInterfaceAccountToken8 | 806           | 🟢 **-11,128 (93.25%)** |
| interfaceAccountMint1       | 217           | 🟢 **-1,259 (85.30%)**  |
| interfaceAccountMint2       | 286           | 🟢 **-2,203 (88.51%)**  |
| interfaceAccountMint4       | 412           | 🟢 **-4,099 (90.87%)**  |
| interfaceAccountMint8       | 669           | 🟢 **-7,881 (92.18%)**  |
| interfaceAccountToken1      | 215           | 🟢 **-1,896 (89.82%)**  |
| interfaceAccountToken2      | 284           | 🟢 **-3,445 (92.38%)**  |
| interfaceAccountToken4      | 407           | 🟢 **-6,548 (94.15%)**  |
| program1                    | 173           | 🟢 **-606 (77.79%)**    |
| program2                    | 200           | 🟢 **-720 (78.26%)**    |
| program4                    | 242           | 🟢 **-951 (79.72%)**    |
| program8                    | 318           | 🟢 **-1,426 (81.77%)**  |
| signer1                     | 167           | 🟢 **-607 (78.42%)**    |
| signer2                     | 191           | 🟢 **-873 (82.05%)**    |
| signer4                     | 233           | 🟢 **-1,404 (85.77%)**  |
| signer8                     | 307           | 🟢 **-2,481 (88.99%)**  |
| systemAccount1              | 173           | 🟢 **-623 (78.27%)**    |
| systemAccount2              | 203           | 🟢 **-893 (81.48%)**    |
| systemAccount4              | 254           | 🟢 **-1,435 (84.96%)**  |
| systemAccount8              | 353           | 🟢 **-2,527 (87.74%)**  |
| uncheckedAccount1           | 164           | 🟢 **-619 (79.05%)**    |
| uncheckedAccount2           | 186           | 🟢 **-870 (82.39%)**    |
| uncheckedAccount4           | 217           | 🟢 **-1,377 (86.39%)**  |
| uncheckedAccount8           | 279           | 🟢 **-2,400 (89.59%)**  |
