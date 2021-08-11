# The register report

*Work in progress*

The register report shows a list of postings. It can be invoked with ```dinero register``` or the shorter ```dinero reg```

# Options

## --collapse

The collapse only shows one posting per account and transaction. For example: 

```
2021-09-01 * A lot of fees
    Expenses:Travel    200 EUR
    Expenses:Fees        1 EUR
    Expenses:Fees        3 EUR
    Assets:Checking Account
```

```dinero reg --collapse``` will print out:

```
2021-09-01  A lot of fees    Expenses:Travel              200 EUR      200 EUR
                             Expenses:Fees                  4 EUR      204 EUR
                             Assets:Checking Account     -204 EUR        0 EUR
```
    
