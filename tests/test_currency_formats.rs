use std::borrow::BorrowMut;
use std::rc::Rc;

use chrono::Utc;
use dinero::models::{Currency, CurrencyDisplayFormat, Money};
use dinero::parser::Tokenizer;
use dinero::{models::conversion, CommonOpts};
use num::traits::Inv;
use num::{BigInt, BigRational};
use structopt::StructOpt;

#[test]
fn currency_formats() {
    let mut tokenizer: Tokenizer = Tokenizer::from(
        "2020-01-01 * ACME, Inc.
    Assets:Shares        1 ACME @ 1000 USD
    Assets:Bank:Checking account
2021-01-01 * ACME, Inc.
    Assets:Shares        1 ACME @ 1000 EUR
    Assets:Bank:Checking account

P 2020-07-01 EUR 1.5 USD
commodity €
    alias EUR
    format -1.234,00 €
commodity $
    alias USD
    format ($1,234.00)
commodity ACME
    format -1 ACME
; I have 2 ACME Shares
; worth 2000 EUR
; worth 3000 USD because the last exchange rate was 1.5
; in terms of nodes there should be
; 2021-01-01 ACME
; 2021-01-01 EUR
; 2020-07-01 EUR
; 2020-07-01 USD
; NOTHING for 2020-01-01
;
        "
        .to_string(),
    );
    let options = CommonOpts::from_iter([""].iter());
    let items = tokenizer.tokenize(&options);
    let ledger = items.to_ledger(&options).unwrap();
    let eur = ledger.get_commodities().get("eur").unwrap();
    let usd = ledger.get_commodities().get("usd").unwrap();
    let acme = ledger.get_commodities().get("acme").unwrap();
    for _ in 0..30 {
        let multipliers_acme = conversion(
            acme.clone(),
            Utc::now().naive_local().date(),
            ledger.get_prices(),
        );

        let to_eur = multipliers_acme.get(eur).unwrap();
        let to_usd = multipliers_acme.get(usd).unwrap();
        assert_eq!(
            to_eur,
            &BigRational::from_integer(BigInt::from(1000)).inv(),
            "1 ACME = 1000 EUR"
        );
        assert_eq!(
            to_usd,
            &BigRational::from_integer(BigInt::from(1500)).inv(),
            "1 ACME = 1500 USD"
        );
    }
}

#[test]
fn display_currencies() {
    let format_1 = CurrencyDisplayFormat::from("-1.234,00 €");
    let format_2 = CurrencyDisplayFormat::from("(1.234,00 €)");
    let format_3 = CurrencyDisplayFormat::from("-€1.234,00 €");

    let currency = Rc::new(Currency::from("€"));
    let money = Money::from((currency.clone(), BigRational::from_float(-12.3).unwrap()));

    currency.set_format(&format_1);
    assert_eq!(format!("{}", money), "-12,30 €");

    // TODO Fix this test
    // currency.set_format(&format_2);
    // assert_eq!(format!("{}", money), "(12,30 €)");

    currency.set_format(&format_3);
    assert_eq!(format!("{}", money), "-€12,30");
}
