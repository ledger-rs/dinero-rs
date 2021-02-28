#!/bin/sh

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cd $DIR

ledger commodities | sort > commodities_ledger.txt &
dinero commodities | sort > commodities_dinero.txt &

ledger payees | sort > payees_ledger.txt &
dinero payees | sort > payees_dinero.txt &

ledger bal stockplan -X eur > bal_stockplan_ledger.txt &
dinero bal stockplan -X eur > bal_stockplan_dinero.txt &

ledger bal ^activo -X eur > bal_activo_ledger.txt &
dinero bal ^activo -X eur > bal_activo_dinero.txt &

