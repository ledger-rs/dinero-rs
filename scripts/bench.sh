#!/bin/sh
echo "This benchmark needs ledgerrc to be properly set up for it to be meaningful"

hyperfine -L command dinero,ledger '{command} bal'
hyperfine -L command dinero,ledger '{command} bal stockplan -X eur'
hyperfine -L command dinero,ledger '{command} bal degiro -X eur'
hyperfine -L command dinero,ledger '{command} bal vactivo -X eur'
