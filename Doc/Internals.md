

Loan concepts:
* LoanLimit: The maximum amount a user can withdraw as loan. It is dependend on the number of assets given as colleteral
* LoanOpen: The current amount of oustanding loan. This amount consists of what was previously withdrawn and its interest
* LoanLastChange: The last blocknumber in which LoanOpen was changed. This value is used to calculate interest on LoanOpen.