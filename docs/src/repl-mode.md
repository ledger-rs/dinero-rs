# REPL mode

Since ```0.29.0``` ```dinero-rs``` comes with a REPL mode (read-eval-print-loop) or interactive mode:

```dinero -f myjournal.ledger````

Once inside the REPL mode, the ledger is parsed and cached so that any subsequent operations are faster than their regular CLI counterparts.

## Working inside the interactive mode

The commands behave just like in the normal mode but:
- they are faster
- the ```dinero``` executable is elided, you can write either ```dinero reg``` or ```reg```

## Special commands

To exit the REPL type ```exit``` or ```quit```.

```reload``` loads the journal again, which is useful if it has been changed externally.
