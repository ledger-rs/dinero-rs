//! Document the command line interface
use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use two_timer;

use lazy_static::lazy_static;
use regex::Regex;

use crate::commands::{accounts, balance, check, commodities, prices, register};
use crate::Error;
use chrono::NaiveDate;
use colored::Colorize;

#[derive(Debug, StructOpt)]
enum Command {
    // Print,
    /// Balance report
    #[structopt(alias = "bal")]
    Balance {
        #[structopt(flatten)]
        options: CommonOpts,
        /// Flat account names rather than tree
        #[structopt(long)]
        flat: bool,
        /// Do not show total
        #[structopt(long = "--no-total")]
        no_total: bool,
    },

    #[structopt(alias = "reg")]
    Register(CommonOpts),
    /// List the accounts
    Accounts(CommonOpts),
    // Codes,
    // Payees,
    /// Show the exchange rates
    Prices(CommonOpts),
    /// List commodities
    #[structopt(alias = "currencies")]
    Commodities(CommonOpts),
    /// Simply check the file is fine
    Check(CommonOpts),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Command line accounting tool",
version = env ! ("CARGO_PKG_VERSION"),
author = env ! ("CARGO_PKG_AUTHORS"),
name = "dinero"
)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
pub struct CommonOpts {
    /// Input file
    #[structopt(name = "FILE", short = "f", long = "file", parse(from_os_str))]
    pub(crate) input_file: PathBuf,

    /// Init file
    #[structopt(long = "--init-file", parse(from_os_str))]
    init_file: Option<PathBuf>,

    /// Depth
    #[structopt(short = "d", long = "depth")]
    pub(crate) depth: Option<usize>,

    /// The pattern to look for
    #[structopt(multiple = true, takes_value = true)]
    pub(crate) query: Vec<String>,
    /// Use only real postings rather than real and virtual
    #[structopt(long = "--real")]
    pub(crate) real: bool,
    #[structopt(short = "b", long = "begin", parse(try_from_str = date_parser))]
    pub(crate) begin: Option<NaiveDate>,
    #[structopt(short = "e", long = "end", parse(try_from_str = date_parser))]
    pub(crate) end: Option<NaiveDate>,
    #[structopt(short = "p", long = "period")]
    period: Option<String>,
    #[structopt(long = "now", parse(try_from_str = date_parser))]
    now: Option<NaiveDate>,

    /// Ignore balance assertions
    #[structopt(long = "--no-balance-check")]
    pub(crate) no_balance_check: bool,

    /// Display the report in the selected currency
    #[structopt(short = "-X")]
    pub(crate) exchange: Option<String>,

    /// TODO Date format
    #[structopt(long = "--date-format")]
    date_format: Option<String>,

    /// TODO force color
    #[structopt(long = "--force-color")]
    force_color: bool,
    /// TODO force pager
    #[structopt(long = "--force-pager")]
    force_pager: bool,

    /// TODO effective
    #[structopt(long = "--effective")]
    effective: bool,
    /// TODO strict
    #[structopt(long = "--strict")]
    strict: bool,

    /// TODO Unrealized gains
    #[structopt(long = "--unrealized-gains")]
    unrealized_gains: Option<String>,
    /// TODO Unrealized losses
    #[structopt(long = "--unrealized-losses")]
    unrealized_losses: Option<String>,
}

pub fn run_app(mut args: Vec<String>) -> Result<(), ()> {
    // println!("{:?}", args);
    let mut possible_paths: Vec<String> = Vec::new();
    for i in 0..args.len() {
        if args[i] == "--init-file" {
            possible_paths.push(args[i + 1].clone());
            break;
        }
    }
    possible_paths.push(shellexpand::tilde("~/.ledgerrc").to_string());
    possible_paths.push(".ledgerrc".to_string());
    let mut config_file = None;
    for path in possible_paths.iter() {
        let file = Path::new(path);
        if file.exists() {
            config_file = Some(file);
            break;
        }
    }
    if let Some(file) = config_file {
        let mut aliases = HashMap::new();
        aliases.insert("-f".to_string(), "--file".to_string());
        let contents = read_to_string(file).unwrap();
        for line in contents.lines() {
            let option = line.trim_start();
            match option.chars().nth(0) {
                Some(c) => match c {
                    '-' => {
                        assert!(
                            line.starts_with("--"),
                            format!("Bad config file {:?}\n{}", file, line)
                        );
                        let mut iter = line.split_whitespace();
                        let option = iter.next().unwrap();
                        if !args.iter().any(|x| {
                            (x == option) | (aliases.get(x).unwrap_or(&String::new()) == option)
                        }) {
                            args.push(option.to_string());
                            let mut rest = String::new();
                            for arg in iter {
                                rest.push_str(" ");
                                rest.push_str(arg);
                            }
                            if rest.len() > 0 {
                                args.push(rest.trim().to_string());
                            }
                        }
                    }
                    ';' | '#' | '!' | '%' => (), // a comment

                    _ => panic!("Bad config file {:?}\n{}", file, line),
                },
                None => (),
            }
        }
    }
    let opt: Opt = Opt::from_iter(args.iter());

    // Print options
    // println!("{:?}", opt.cmd);
    if let Err(e) = match opt.cmd {
        Command::Balance {
            options,
            flat,
            no_total,
        } => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }
            balance::execute(&options, flat, !no_total)
        }
        Command::Register(options) => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }
            register::execute(&options)
        }
        Command::Commodities(options) => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }

            commodities::execute(options.input_file, options.no_balance_check)
        }
        Command::Prices(options) => prices::execute(options.input_file, options.no_balance_check),
        Command::Accounts(options) => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }

            accounts::execute(options.input_file, options.no_balance_check)
        }
        Command::Check(options) => check::execute(options.input_file),
    } {
        let err_str = format!("{}", e);
        if err_str.len() > 0 {eprintln!("{}", err_str);}
        return Err(());
    }
    Ok(())
}

/// A parser for date expressions
fn date_parser(date: &str) -> Result<NaiveDate, Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(\d{4})[/-](\d\d?)$").unwrap();
    }
    if RE.is_match(date) {
        let captures = RE.captures(date).unwrap();
        Ok(NaiveDate::from_ymd(
            captures.get(1).unwrap().as_str().parse::<i32>().unwrap(),
            captures.get(2).unwrap().as_str().parse::<u32>().unwrap(),
            1,
        ))
    } else {
        match two_timer::parse(date, None) {
            Ok((t1, _t2, _b)) => Ok(t1.date()),
            Err(e) => {
                eprintln!("{:?}", e);
                Err(Error {
                    message: vec![format!("Invalid date {}", date)
                        .as_str()
                        .bold()
                        .bright_red()],
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_line_dates() {
        assert_eq!(
            date_parser("2010-5-3").unwrap(),
            NaiveDate::from_ymd(2010, 5, 3)
        );
        assert_eq!(
            date_parser("2010").unwrap(),
            NaiveDate::from_ymd(2010, 1, 1)
        );
        assert_eq!(
            date_parser("2010-09").unwrap(),
            NaiveDate::from_ymd(2010, 9, 1)
        );
        assert!(date_parser("this is not a date").is_err());
    }

    #[test]
    fn test_balance() {
        let args: Vec<String> = vec![
            "testing",
            "bal",
            "-f",
            "examples/demo.ledger",
            "--init-file",
            "examples/example_ledgerrc",
            "--real",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let res = run_app(args);
        assert!(res.is_ok());
    }

    #[test]
    #[should_panic(
        expected = "Bad config file \"examples/example_bad_ledgerrc\"\nThis line should be a comment but isn\'t, it is bad on purpose."
    )]
    fn bad_ledgerrc() {
        let args: Vec<String> = vec![
            "testing",
            "bal",
            "--init-file",
            "examples/example_bad_ledgerrc",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let _res = run_app(args);
    }
    #[test]
    #[should_panic(
        expected = "Bad config file \"examples/example_bad_ledgerrc2\"\n- This does not parse either. And it shouldn't."
    )]
    fn other_bad_ledgerrc() {
        let args: Vec<String> = vec![
            "testing",
            "bal",
            "--init-file",
            "examples/example_bad_ledgerrc2",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let _res = run_app(args);
    }
    #[test]
    #[should_panic]
    fn file_does_not_exist() {
        let args: Vec<String> = vec!["testing", "bal", "-f", "this_file_does_not_exist.ledger"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let _res = run_app(args);
    }
}
