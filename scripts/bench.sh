#!/bin/sh
echo "This benchmark needs ledgerrc to be properly set up for it to be meaningful"

hyperfine 'dinero bal'
hyperfine 'ledger bal'

