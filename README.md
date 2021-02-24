[![Build Status](https://travis-ci.com/frosklis/dinero-rs.svg?branch=master)](https://travis-ci.com/frosklis/dinero-rs)
[![codecov](https://codecov.io/gh/frosklis/dinero-rs/branch/master/graph/badge.svg?token=QC4LG2ZMZJ)](https://codecov.io/gh/frosklis/dinero-rs)
[![crates.io](https://img.shields.io/crates/v/dinero-rs)](https://crates.io/crates/dinero-rs)
![Crates.io](https://img.shields.io/crates/l/dinero-rs)

Dinero (spanish for money) is a command line tool that can deal with ledger files, as defined by John Wiegley's wonderful [ledger-cli](https://www.ledger-cli.org/).

# Quickstart

## Install

If you have Rust available, the easiest way to get dinero-rs is by install the crate:
```sh
cargo install dinero-rs
```
- [ ] Installation for Windows
- [ ] Installation for Mac
- [ ] Installation for Linux

## First steps

Dinero uses double entry accounting. Store your journal files in ```ledger``` files. The main item is a transaction, which in its basic form looks something like this:

```ledger
; This is a comment
; A date followed by a description identifies the beginning of a transaction
2021-02-01 Buy fruit
     Expenses:Groceries          7.92 EUR
     Assets:Checking account             ; you can leave this blank, dinero balances the transactions for you
```

After that, you can issue all the commands you want and combine them with options to have complete control over your finances!

The most basic ones are:
```sh
# Get a balance report: How much is there in all 
dinero bal -f myledger.ledger

# Get a list of transactions
dinero reg -f myledger.ledger
```

# Features

Currently supported are:
- Balance reports
- Register reports

Report filtering by account name and by date.

# Motivation
I use ledger-cli extensively for my personal finances. My goal is to be able to run all the commands I use the most with my own tool while at the same time learning Rust.

Run ```dinero --help``` for a list of available commands and options.
