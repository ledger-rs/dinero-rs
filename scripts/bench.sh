#!/bin/sh
echo "This benchmark needs ledgerrc to be properly set up for it to be meaningful"

hyperfine 'ledger bal'

hyperfine 'dinero bal'
