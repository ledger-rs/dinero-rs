# Changelog
Changelog file for dinero-rs project, a command line application for managing finances.

## [0.17.0] - planned
### Added
### Changed
- Now the whole file is processed using a formal grammar

### Fixed
- Now this can be done ```any(abs(amount) == 2)```, which failed previously
- Much faster CI builds
- Proper caching of regexes, [about 25% speed improvement](https://github.com/frosklis/dinero-rs/issues/40)
## [0.16.0] - 2021-03-04
### Added
- Virtual postings show correctly like this ```(account)```
### Fixed 
- Now you can add tags [through automated transactions](https://github.com/frosklis/dinero-rs/issues/49)
## [0.15.0] - 2021-02-28
### Fixed
- Correct conversion of currencies. There were [certain cases that did not work properly](https://github.com/frosklis/dinero-rs/issues/37)
### Added
- complete transaction grammar
## [0.14.0] - 2021-02-27
### Fixed
- speed bump, from 7 seconds to 4 seconds in my personal ledger (still room to improve)
- ability to add tags from automated transactions
## [0.13.1] - 2021-02-27 
### Fixed
- Fixed issue when there is no specified payee
## [0.13.0] - 2021-02-27 
### Added
- Improved documentation
- Support for [hledger syntax for payees](https://github.com/frosklis/dinero-rs/issues/37)
### Fixed
- keep tags from transactions
- match automated transactions only once per transaction, like ```ledger``` does
- enable comments in price ```p``` directives
## [0.12.0] - 2021-02-24
### Added
- support for (some of the) automated transaction syntax, what Claudio uses in his personal ledger
### Fixed
- speed bump (from 44 seconds to 7 seconds) in a big personal ledger

## [0.11.1] - 2021-02-22
### Fixed
- Fixed bug in balance report
