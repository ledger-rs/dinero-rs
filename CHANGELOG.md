# Changelog
Changelog file for dinero-rs project, a command line application for managing finances.
## [0.33.0] - xxx
## Added
- Show internal return rate with the ```roi``` command
## Changed
- Nicer error messages (without Rust trace) when there is a missing file.
## Fixed
- [Rounding](https://github.com/frosklis/dinero-rs/issues/142)
## [0.32.3] - 2021-08-24
- The last one was a bad release
## [0.32.2] - 2021-08-24 
### Fixed
- Now [parameters can be overriden](https://github.com/frosklis/dinero-rs/issues/138)

## [0.32.1] - 2021-08-24
### Changed
- continuous integration pipeline
## [0.32.0] - 2021-08-24
### Added
- Implemented ```date-format```
- Added ```--calendar``` to the ```roi``` command, showing a [calendar view of TWR](https://github.com/frosklis/dinero-rs/issues/115).
- Added ```--no-summary``` flag to the ```roi``` command, to suppress the summary after the table
- Implemented [```--related``` flag](https://github.com/frosklis/dinero-rs/issues/102)
### Fixed
- [```args-only```](https://github.com/frosklis/dinero-rs/issues/120)

## [0.31.0] - 2021-08-22 
### Added
- [The ```roi``` command](https://github.com/frosklis/dinero-rs/issues/115) is good enough

### Fixed
- [Currencies are shown consistently in a report](https://github.com/frosklis/dinero-rs/issues/103)
- Read quantities like ```-$0.25```, [bug](https://github.com/frosklis/dinero-rs/issues/126)

## [0.30.0] - 2021-08-18
## Added
- Show more info when loading the repl
- Ability to [reload the journal](https://github.com/frosklis/dinero-rs/issues/116) 
## Fixed
- [```Some payees were None```](https://github.com/frosklis/dinero-rs/issues/121)

## [0.29.1] - 2021-08-17
## Changed
- small improvements on REPL interface
- improved test coverage
## [0.29.0] - 2021-08-16
### Added
- ```exchange``` option (```-X```) for register reports
- REPL interface, which is faster than the CLI once everything's loaded
### Changed
- Some internal tests now use the ```--init-file``` flag to make sure the environment is properly replicated.
- Updated dependency from ```assert_cmd to 2.0```

## [0.28.1] - 2021-08-10
### Fixed
- The previous crate was created badly.

## [0.28.0] - 2021-08-09
### Added
- ```--collapse``` flag to collapse postings with the same currency and account
## [0.27.0] - 2021-08-04
### Fixed
- Negative quantities starting with zero now show the negative sign.
## [0.26.0] - 2021-08-02
### Added
- ```--args-only``` flag to ignore init files
- ```precision``` property in the ```commodity``` directive
### Changed
- Check whether dependencies are updated or not with deps.rs service
### Fixed
- [```--strict``` and ```--pedantic``` working properly](https://github.com/frosklis/dinero-rs/issues/104)
## [0.25.0] - 2021-03-31
### Added
- nicer error reporting
- slightly better documentation
- [```stats``` command](https://github.com/frosklis/dinero-rs/issues/96) that shows statistics about your ledger file
### Fixed
- No need to [add a space before ```=``` in balance assertions](https://github.com/frosklis/dinero-rs/issues/40)
- Correct parsing of transaction codes
## [0.24.0] - 2021-03-29
### Added
- ```strict``` and ```pedantic``` options
### Changed
- Collaborators will be able to use codecov as well
## [0.23.0] - 2021-03-24
### Added
- Accounts now have a ```country``` property
- Documentation is now available at github.
### Changed
- Accounts no longer support ```isin``` property. They do support ```iban```, which is what should have always been.
- Migrated the CI pipeline to Github Actions because I had trouble with Travis (build matrices)

## [0.22.0] - 2021-03-21
### Added
- Slightly better handling of currency formats
### Changed
- Better CI pipeline

## [0.21.0] - 2021-03-20
### Added
- Infer currency format from the journal file
- ```isin``` is a valid property for commodities
### Changed
- Continuous integration pipeline is now better. No more problems like what happened between releases 0.18 and 0.20.
### Fixed
- Commodities get parsed properly, always removing quotes
## [0.20.0] - 2021-03-15
### Fixed
- Version numbers back on track
## [0.19.0] - 2021-03-15
- Same as 0.18.1 due to a mistake
## [0.18.1] - 2021-03-15
### Fixed
- Don't panic on end of input
## [0.18.0] - 2021-03-14
### Added
- Support for specifying payees via posting comments.
- Added support for dates in posting comments
- Added support for specifying currency formats
### Changed
- Date comparisons are done at the posting level rather than the transaction level
## [0.17.0] - 2021-03-12
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
