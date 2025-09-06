# Compute Units

(Using anchor's bench to generate with star frame's bench.so)

## [Unreleased]

Solana version: 2.1.0

| Instruction                 | Compute Units | -                       |
| --------------------------- | ------------- | ----------------------- |
| accountInfo1                | 225           | 🟢 **-346 (60.60%)**    |
| accountInfo2                | 266           | 🟢 **-629 (70.28%)**    |
| accountInfo4                | 338           | 🟢 **-1,215 (78.24%)**  |
| accountInfo8                | 484           | 🟢 **-2,439 (83.44%)**  |
| accountEmptyInit1           | 2,218         | 🟢 **-2,865 (56.36%)**  |
| accountEmpty1               | 288           | 🟢 **-357 (55.35%)**    |
| accountEmptyInit2           | 3,922         | 🟢 **-5,379 (57.83%)**  |
| accountEmpty2               | 396           | 🟢 **-611 (60.68%)**    |
| accountEmptyInit4           | 7,339         | 🟢 **-10,425 (58.69%)** |
| accountEmpty4               | 595           | 🟢 **-1,129 (65.49%)**  |
| accountEmptyInit8           | 14,160        | 🟢 **-20,563 (59.22%)** |
| accountEmpty8               | 1,005         | 🟢 **-2,158 (68.23%)**  |
| accountSizedInit1           | 2,221         | 🟢 **-2,971 (57.22%)**  |
| accountSized1               | 290           | 🟢 **-403 (58.15%)**    |
| accountSizedInit2           | 3,926         | 🟢 **-5,563 (58.63%)**  |
| accountSized2               | 397           | 🟢 **-678 (63.07%)**    |
| accountSizedInit4           | 7,346         | 🟢 **-10,824 (59.57%)** |
| accountSized4               | 595           | 🟢 **-1,253 (67.80%)**  |
| accountSizedInit8           | 14,176        | 🟢 **-21,257 (59.99%)** |
| accountSized8               | 1,004         | 🟢 **-2,383 (70.36%)**  |
| accountUnsizedInit1         | 2,240         | 🟢 **-3,065 (57.78%)**  |
| accountUnsized1             | 288           | 🟢 **-458 (61.39%)**    |
| accountUnsizedInit2         | 3,963         | 🟢 **-5,796 (59.39%)**  |
| accountUnsized2             | 397           | 🟢 **-766 (65.86%)**    |
| accountUnsizedInit4         | 7,421         | 🟢 **-11,182 (60.11%)** |
| accountUnsized4             | 595           | 🟢 **-1,407 (70.28%)**  |
| accountUnsizedInit8         | 14,327        | 🟢 **-21,666 (60.20%)** |
| accountUnsized8             | 1,005         | 🟢 **-2,668 (72.64%)**  |
| boxedAccountEmptyInit1      | 2,248         | 🟢 **-2,927 (56.56%)**  |
| boxedAccountEmpty1          | 302           | 🟢 **-432 (58.86%)**    |
| boxedAccountEmptyInit2      | 3,982         | 🟢 **-5,432 (57.70%)**  |
| boxedAccountEmpty2          | 425           | 🟢 **-691 (61.92%)**    |
| boxedAccountEmptyInit4      | 7,459         | 🟢 **-10,459 (58.37%)** |
| boxedAccountEmpty4          | 655           | 🟢 **-1,217 (65.01%)**  |
| boxedAccountEmptyInit8      | 14,400        | 🟢 **-20,553 (58.80%)** |
| boxedAccountEmpty8          | 1,124         | 🟢 **-2,277 (66.95%)**  |
| boxedAccountSizedInit1      | 2,250         | 🟢 **-3,021 (57.31%)**  |
| boxedAccountSized1          | 301           | 🟢 **-482 (61.56%)**    |
| boxedAccountSizedInit2      | 3,987         | 🟢 **-5,596 (58.40%)**  |
| boxedAccountSized2          | 424           | 🟢 **-766 (64.37%)**    |
| boxedAccountSizedInit4      | 7,467         | 🟢 **-10,763 (59.04%)** |
| boxedAccountSized4          | 656           | 🟢 **-1,340 (67.13%)**  |
| boxedAccountSizedInit8      | 14,415        | 🟢 **-21,138 (59.45%)** |
| boxedAccountSized8          | 1,125         | 🟢 **-2,503 (68.99%)**  |
| boxedAccountUnsizedInit1    | 2,269         | 🟢 **-3,102 (57.75%)**  |
| boxedAccountUnsized1        | 301           | 🟢 **-535 (64.00%)**    |
| boxedAccountUnsizedInit2    | 4,024         | 🟢 **-5,735 (58.77%)**  |
| boxedAccountUnsized2        | 425           | 🟢 **-845 (66.54%)**    |
| boxedAccountUnsizedInit4    | 7,543         | 🟢 **-11,015 (59.35%)** |
| boxedAccountUnsized4        | 655           | 🟢 **-1,477 (69.28%)**  |
| boxedAccountUnsizedInit8    | 14,567        | 🟢 **-21,618 (59.74%)** |
| boxedAccountUnsized8        | 1,124         | 🟢 **-2,757 (71.04%)**  |
| boxedInterfaceAccountMint1  | 350           | 🟢 **-1,001 (74.09%)**  |
| boxedInterfaceAccountMint2  | 520           | 🟢 **-1,603 (75.51%)**  |
| boxedInterfaceAccountMint4  | 839           | 🟢 **-2,817 (77.05%)**  |
| boxedInterfaceAccountMint8  | 1,482         | 🟢 **-5,256 (78.01%)**  |
| boxedInterfaceAccountToken1 | 349           | 🟢 **-1,662 (82.65%)**  |
| boxedInterfaceAccountToken2 | 517           | 🟢 **-2,914 (84.93%)**  |
| boxedInterfaceAccountToken4 | 835           | 🟢 **-5,425 (86.66%)**  |
| boxedInterfaceAccountToken8 | 1,474         | 🟢 **-10,460 (87.65%)** |
| interfaceAccountMint1       | 338           | 🟢 **-1,138 (77.10%)**  |
| interfaceAccountMint2       | 489           | 🟢 **-2,000 (80.35%)**  |
| interfaceAccountMint4       | 778           | 🟢 **-3,733 (82.75%)**  |
| interfaceAccountMint8       | 1,363         | 🟢 **-7,187 (84.06%)**  |
| interfaceAccountToken1      | 336           | 🟢 **-1,775 (84.08%)**  |
| interfaceAccountToken2      | 487           | 🟢 **-3,242 (86.94%)**  |
| interfaceAccountToken4      | 773           | 🟢 **-6,182 (88.89%)**  |
| program1                    | 236           | 🟢 **-543 (69.70%)**    |
| program2                    | 285           | 🟢 **-635 (69.02%)**    |
| program4                    | 370           | 🟢 **-823 (68.99%)**    |
| program8                    | 546           | 🟢 **-1,198 (68.69%)**  |
| signer1                     | 227           | 🟢 **-547 (70.67%)**    |
| signer2                     | 272           | 🟢 **-792 (74.44%)**    |
| signer4                     | 352           | 🟢 **-1,285 (78.50%)**  |
| signer8                     | 517           | 🟢 **-2,271 (81.46%)**  |
| systemAccount1              | 251           | 🟢 **-545 (68.47%)**    |
| systemAccount2              | 321           | 🟢 **-775 (70.71%)**    |
| systemAccount4              | 446           | 🟢 **-1,243 (73.59%)**  |
| systemAccount8              | 708           | 🟢 **-2,172 (75.42%)**  |
| uncheckedAccount1           | 224           | 🟢 **-559 (71.39%)**    |
| uncheckedAccount2           | 267           | 🟢 **-789 (74.72%)**    |
| uncheckedAccount4           | 340           | 🟢 **-1,254 (78.67%)**  |
| uncheckedAccount8           | 485           | 🟢 **-2,194 (81.90%)**  |
