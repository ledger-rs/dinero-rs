# The journal file(s)

*Work in progress*

Dinero derives its reports from a *journal* file ([or files](#include_directive)). The most important feature of this file format is its readibility. Unlike other computer-friendly formats such as comma separated values or a binary database, journal files actually make sense to a human.

Dinero follows the principles of [double entry accounting](https://en.wikipedia.org/wiki/Double-entry_bookkeeping), where the main information is the *transaction*.

A transaction contains two or more *postings*, which are actual movements in an *account*, which is another important concept. In bookkeeping, money always comes from and goes to an account.

## Developers

The full syntax accepted by ```dinero```can be found in the [grammar specification](https://github.com/frosklis/dinero-rs/blob/master/src/grammar/grammar.pest). It is a formal grammar.