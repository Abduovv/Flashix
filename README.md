## ⚡ Flashix — Flash Loan Protocol on Solana

Flashix is a lightweight flash loan protocol on Solana that allows anyone to borrow tokens without collateral, as long as they repay within the same transaction. Liquidity providers (LPs) can deposit tokens to earn a share of the fees collected from borrowers.

![Solana](https://img.shields.io/badge/Solana-Devnet-3ECF8E?logo=solana&logoColor=white)
![Anchor](https://img.shields.io/badge/Anchor-Framework-blueviolet)



<div align="center">
  <img src="./flashixlogo.jpg" alt="Banner" width="600"/>
</div>

---

```scss
 Liquidity Provider (LP)
        │
        └──▶ deposit_liquidity
                    │
                    ▼
            [Vault is funded]
                    │
         ┌──────────┴──────────┐
         ▼                     ▼
 Borrower                  LP holds LP tokens
        │
        └──▶ request_flash_loan (amount)
                    │
                    ▼
       [Vault transfers amount to borrower]
                    │
        borrower executes arbitrary logic
                    │
        ┌────────────┴────────────┐
        ▼                         ▼
repay_loan_with_fee       (Fails: TX reverts)
        │
        ▼
[Vault receives amount + fee]
        │
        ▼
[Update vault.total + collected_fees]
        │
        ▼
   LP wants to exit position
        │
        └──▶ withdraw_with_fees
                    │
                    ▼
[Calculate LP share + fee share]
        │
        ▼
[Transfer funds & burn LP tokens]
```
