# Differences with ledger-cli

Although dinero is completely inspired by [```ledger-cli```](https://ledger-cli.org) and implements a subset of its features, it has been written form scratch, it is not a port. 

Some behaviors are intentionally different. Other things are just bugs: if you find one, feel free to add an issue in the [development repository](https://github.com/frosklis/dinero-rs).


```dinero``` is developed in Rust, while ```ledger``` is developed in C. This is completely transparent to the end user, it does at least theoretically provide some advantages for developers, with Rust being a newer language with the same speed as C but more memory safety. Again, this at least theory

The next table presents a summary of differences, with the most important ones being commented later.

What | ledger | dinero
-----|--------|-------
Programming language | C | Rust
Feature set | a lot of options for each command | just some options for each command
Transaction sorting for balance assertion | within file? | global
Speed | extremely fast | not quite as fast (yet)
End with newline | files must end with a blank line | no need to do that
Regular expressions | assume ignore case | not always (it is well known when)
Unicode | not everywhere | € is a valid currency



## Balance assertions

Balance assertions have the same syntax in both languages, but the way they are handled is different.

In ```ledger``` it is very difficult (for me) to add balance assertions to all the transactions, in particular when you have several ledger files linked together. The balance assertions are processed more or less as they appear in the files, which depends in the order you read the files with the ```include``` directive.

In ```dinero``` every transaction is read, then they are sorted by date (without altering the original order in ties) and finally tha balance is checked.

The practical consequence for me (Claudio) in particular is that rather than doing this:
```ledger
include past/201701.ledger
include past/201702.ledger
; ...
include past/202103.ledger
```

I can do this instead:
```ledger
include past/*.ledger
```

This results in a much shorter file, easier on the eyes (I like my master.ledger file to be complete yet simple). ```ledger``` does not guarantee the order in which the files are read and that affects balance assertions. ```dinero``` doesn't guarantee it either, but the extra ordering step means it doesn't matter (though arguably it makes it somewhat slower)

