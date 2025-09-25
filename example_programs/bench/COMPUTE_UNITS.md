# Compute Units

(Using anchor's bench to generate with star frame's bench.so)

## [Unreleased]

Solana version: 2.1.0

| Instruction                 | Compute Units | -                       |
| --------------------------- | ------------- | ----------------------- |
| accountInfo1                | 166           | 🟢 **-405 (70.93%)**    |
| accountInfo2                | 181           | 🟢 **-714 (79.78%)**    |
| accountInfo4                | 216           | 🟢 **-1,337 (86.09%)**  |
| accountInfo8                | 277           | 🟢 **-2,646 (90.52%)**  |
| accountEmptyInit1           | 1,984         | 🟢 **-3,099 (60.97%)**  |
| accountEmpty1               | 189           | 🟢 **-456 (70.70%)**    |
| accountEmptyInit2           | 3,582         | 🟢 **-5,719 (61.49%)**  |
| accountEmpty2               | 231           | 🟢 **-776 (77.06%)**    |
| accountEmptyInit4           | 6,785         | 🟢 **-10,979 (61.80%)** |
| accountEmpty4               | 319           | 🟢 **-1,405 (81.50%)**  |
| accountEmptyInit8           | 13,178        | 🟢 **-21,545 (62.05%)** |
| accountEmpty8               | 481           | 🟢 **-2,682 (84.79%)**  |
| accountSizedInit1           | 1,983         | 🟢 **-3,209 (61.81%)**  |
| accountSized1               | 191           | 🟢 **-502 (72.44%)**    |
| accountSizedInit2           | 3,578         | 🟢 **-5,911 (62.29%)**  |
| accountSized2               | 232           | 🟢 **-843 (78.42%)**    |
| accountSizedInit4           | 6,776         | 🟢 **-11,394 (62.71%)** |
| accountSized4               | 319           | 🟢 **-1,529 (82.74%)**  |
| accountSizedInit8           | 13,162        | 🟢 **-22,271 (62.85%)** |
| accountSized8               | 480           | 🟢 **-2,907 (85.83%)**  |
| accountUnsizedInit1         | 1,983         | 🟢 **-3,322 (62.62%)**  |
| accountUnsized1             | 189           | 🟢 **-557 (74.66%)**    |
| accountUnsizedInit2         | 3,577         | 🟢 **-6,182 (63.35%)**  |
| accountUnsized2             | 232           | 🟢 **-931 (80.05%)**    |
| accountUnsizedInit4         | 6,775         | 🟢 **-11,828 (63.58%)** |
| accountUnsized4             | 319           | 🟢 **-1,683 (84.07%)**  |
| accountUnsizedInit8         | 13,161        | 🟢 **-22,832 (63.43%)** |
| accountUnsized8             | 481           | 🟢 **-3,192 (86.90%)**  |
| boxedAccountEmptyInit1      | 2,004         | 🟢 **-3,171 (61.28%)**  |
| boxedAccountEmpty1          | 205           | 🟢 **-529 (72.07%)**    |
| boxedAccountEmptyInit2      | 3,621         | 🟢 **-5,793 (61.54%)**  |
| boxedAccountEmpty2          | 265           | 🟢 **-851 (76.25%)**    |
| boxedAccountEmptyInit4      | 6,862         | 🟢 **-11,056 (61.70%)** |
| boxedAccountEmpty4          | 390           | 🟢 **-1,482 (79.17%)**  |
| boxedAccountEmptyInit8      | 13,331        | 🟢 **-21,622 (61.86%)** |
| boxedAccountEmpty8          | 633           | 🟢 **-2,768 (81.39%)**  |
| boxedAccountSizedInit1      | 2,002         | 🟢 **-3,269 (62.02%)**  |
| boxedAccountSized1          | 204           | 🟢 **-579 (73.95%)**    |
| boxedAccountSizedInit2      | 3,618         | 🟢 **-5,965 (62.25%)**  |
| boxedAccountSized2          | 264           | 🟢 **-926 (77.82%)**    |
| boxedAccountSizedInit4      | 6,854         | 🟢 **-11,376 (62.40%)** |
| boxedAccountSized4          | 391           | 🟢 **-1,605 (80.41%)**  |
| boxedAccountSizedInit8      | 13,314        | 🟢 **-22,239 (62.55%)** |
| boxedAccountSized8          | 634           | 🟢 **-2,994 (82.52%)**  |
| boxedAccountUnsizedInit1    | 2,002         | 🟢 **-3,369 (62.73%)**  |
| boxedAccountUnsized1        | 204           | 🟢 **-632 (75.60%)**    |
| boxedAccountUnsizedInit2    | 3,617         | 🟢 **-6,142 (62.94%)**  |
| boxedAccountUnsized2        | 265           | 🟢 **-1,005 (79.13%)**  |
| boxedAccountUnsizedInit4    | 6,854         | 🟢 **-11,704 (63.07%)** |
| boxedAccountUnsized4        | 390           | 🟢 **-1,742 (81.71%)**  |
| boxedAccountUnsizedInit8    | 13,314        | 🟢 **-22,871 (63.21%)** |
| boxedAccountUnsized8        | 633           | 🟢 **-3,248 (83.69%)**  |
| boxedInterfaceAccountMint1  | 231           | 🟢 **-1,120 (82.90%)**  |
| boxedInterfaceAccountMint2  | 315           | 🟢 **-1,808 (85.16%)**  |
| boxedInterfaceAccountMint4  | 481           | 🟢 **-3,175 (86.84%)**  |
| boxedInterfaceAccountMint8  | 808           | 🟢 **-5,930 (88.01%)**  |
| boxedInterfaceAccountToken1 | 230           | 🟢 **-1,781 (88.56%)**  |
| boxedInterfaceAccountToken2 | 312           | 🟢 **-3,119 (90.91%)**  |
| boxedInterfaceAccountToken4 | 477           | 🟢 **-5,783 (92.38%)**  |
| boxedInterfaceAccountToken8 | 800           | 🟢 **-11,134 (93.30%)** |
| interfaceAccountMint1       | 217           | 🟢 **-1,259 (85.30%)**  |
| interfaceAccountMint2       | 280           | 🟢 **-2,209 (88.75%)**  |
| interfaceAccountMint4       | 408           | 🟢 **-4,103 (90.96%)**  |
| interfaceAccountMint8       | 662           | 🟢 **-7,888 (92.26%)**  |
| interfaceAccountToken1      | 215           | 🟢 **-1,896 (89.82%)**  |
| interfaceAccountToken2      | 278           | 🟢 **-3,451 (92.54%)**  |
| interfaceAccountToken4      | 403           | 🟢 **-6,552 (94.21%)**  |
| program1                    | 174           | 🟢 **-605 (77.66%)**    |
| program2                    | 196           | 🟢 **-724 (78.70%)**    |
| program4                    | 242           | 🟢 **-951 (79.72%)**    |
| program8                    | 322           | 🟢 **-1,422 (81.54%)**  |
| signer1                     | 168           | 🟢 **-606 (78.29%)**    |
| signer2                     | 187           | 🟢 **-877 (82.42%)**    |
| signer4                     | 232           | 🟢 **-1,405 (85.83%)**  |
| signer8                     | 311           | 🟢 **-2,477 (88.85%)**  |
| systemAccount1              | 174           | 🟢 **-622 (78.14%)**    |
| systemAccount2              | 199           | 🟢 **-897 (81.84%)**    |
| systemAccount4              | 254           | 🟢 **-1,435 (84.96%)**  |
| systemAccount8              | 357           | 🟢 **-2,523 (87.60%)**  |
| uncheckedAccount1           | 166           | 🟢 **-617 (78.80%)**    |
| uncheckedAccount2           | 183           | 🟢 **-873 (82.67%)**    |
| uncheckedAccount4           | 217           | 🟢 **-1,377 (86.39%)**  |
| uncheckedAccount8           | 279           | 🟢 **-2,400 (89.59%)**  |
