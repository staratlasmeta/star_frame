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
| accountEmptyInit1           | 1,984         | 🟢 **-3,099 (60.97%)**  |
| accountEmpty1               | 188           | 🟢 **-457 (70.85%)**    |
| accountEmptyInit2           | 3,582         | 🟢 **-5,719 (61.49%)**  |
| accountEmpty2               | 235           | 🟢 **-772 (76.66%)**    |
| accountEmptyInit4           | 6,785         | 🟢 **-10,979 (61.80%)** |
| accountEmpty4               | 316           | 🟢 **-1,408 (81.67%)**  |
| accountEmptyInit8           | 13,178        | 🟢 **-21,545 (62.05%)** |
| accountEmpty8               | 478           | 🟢 **-2,685 (84.89%)**  |
| accountSizedInit1           | 1,990         | 🟢 **-3,202 (61.67%)**  |
| accountSized1               | 190           | 🟢 **-503 (72.58%)**    |
| accountSizedInit2           | 3,592         | 🟢 **-5,897 (62.15%)**  |
| accountSized2               | 236           | 🟢 **-839 (78.05%)**    |
| accountSizedInit4           | 6,804         | 🟢 **-11,366 (62.55%)** |
| accountSized4               | 316           | 🟢 **-1,532 (82.90%)**  |
| accountSizedInit8           | 13,218        | 🟢 **-22,215 (62.70%)** |
| accountSized8               | 477           | 🟢 **-2,910 (85.92%)**  |
| accountUnsizedInit1         | 1,990         | 🟢 **-3,315 (62.49%)**  |
| accountUnsized1             | 188           | 🟢 **-558 (74.80%)**    |
| accountUnsizedInit2         | 3,591         | 🟢 **-6,168 (63.20%)**  |
| accountUnsized2             | 236           | 🟢 **-927 (79.71%)**    |
| accountUnsizedInit4         | 6,803         | 🟢 **-11,800 (63.43%)** |
| accountUnsized4             | 316           | 🟢 **-1,686 (84.22%)**  |
| accountUnsizedInit8         | 13,217        | 🟢 **-22,776 (63.28%)** |
| accountUnsized8             | 478           | 🟢 **-3,195 (86.99%)**  |
| boxedAccountEmptyInit1      | 2,004         | 🟢 **-3,171 (61.28%)**  |
| boxedAccountEmpty1          | 206           | 🟢 **-528 (71.93%)**    |
| boxedAccountEmptyInit2      | 3,621         | 🟢 **-5,793 (61.54%)**  |
| boxedAccountEmpty2          | 271           | 🟢 **-845 (75.72%)**    |
| boxedAccountEmptyInit4      | 6,861         | 🟢 **-11,057 (61.71%)** |
| boxedAccountEmpty4          | 387           | 🟢 **-1,485 (79.33%)**  |
| boxedAccountEmptyInit8      | 13,330        | 🟢 **-21,623 (61.86%)** |
| boxedAccountEmpty8          | 630           | 🟢 **-2,771 (81.48%)**  |
| boxedAccountSizedInit1      | 2,009         | 🟢 **-3,262 (61.89%)**  |
| boxedAccountSized1          | 205           | 🟢 **-578 (73.82%)**    |
| boxedAccountSizedInit2      | 3,632         | 🟢 **-5,951 (62.10%)**  |
| boxedAccountSized2          | 270           | 🟢 **-920 (77.31%)**    |
| boxedAccountSizedInit4      | 6,881         | 🟢 **-11,349 (62.25%)** |
| boxedAccountSized4          | 388           | 🟢 **-1,608 (80.56%)**  |
| boxedAccountSizedInit8      | 13,369        | 🟢 **-22,184 (62.40%)** |
| boxedAccountSized8          | 631           | 🟢 **-2,997 (82.61%)**  |
| boxedAccountUnsizedInit1    | 2,009         | 🟢 **-3,362 (62.60%)**  |
| boxedAccountUnsized1        | 205           | 🟢 **-631 (75.48%)**    |
| boxedAccountUnsizedInit2    | 3,631         | 🟢 **-6,128 (62.79%)**  |
| boxedAccountUnsized2        | 271           | 🟢 **-999 (78.66%)**    |
| boxedAccountUnsizedInit4    | 6,881         | 🟢 **-11,677 (62.92%)** |
| boxedAccountUnsized4        | 387           | 🟢 **-1,745 (81.85%)**  |
| boxedAccountUnsizedInit8    | 13,369        | 🟢 **-22,816 (63.05%)** |
| boxedAccountUnsized8        | 630           | 🟢 **-3,251 (83.77%)**  |
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
