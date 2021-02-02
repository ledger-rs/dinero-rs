[![Build Status](https://travis-ci.com/frosklis/dinero-rs.svg?branch=master)](https://travis-ci.com/frosklis/dinero-rs)
[![codecov](https://codecov.io/gh/frosklis/dinero-rs/branch/master/graph/badge.svg?token=QC4LG2ZMZJ)](https://codecov.io/gh/frosklis/dinero-rs)
[![crates.io](https://img.shields.io/crates/v/dinero-rs)](https://crates.io/crates/dinero-rs)
![Crates.io](https://img.shields.io/crates/l/dinero-rs)

Dinero (spanish for money) is a command line tool that can deal with ledger files, as defined by John Wiegley's wonderful [ledger-cli](https://www.ledger-cli.org/).

I use ledger-cli extensively for my personal finances. My goal is to be able to run all the commands I use the most with my own tool while at the same time learning Rust.

Currently supported are:
- Balance reports
- Register reports

Report filtering by account name and by date.

Run ```dinero --help``` for a list of available commands and options.
