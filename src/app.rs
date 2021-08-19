//! Document the command line interface
use shlex::Shlex;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Instant;
use structopt::StructOpt;
use two_timer;

use lazy_static::lazy_static;
use regex::Regex;

use crate::commands::{accounts, balance, commodities, payees, prices, register, roi, statistics};
use crate::models::Ledger;
use crate::Error;
use chrono::NaiveDate;
use colored::Colorize;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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
    /// List the payees
    Payees(CommonOpts),
    /// Show the exchange rates
    Prices(CommonOpts),
    /// List commodities
    #[structopt(alias = "currencies")]
    Commodities(CommonOpts),
    /// List commodities
    #[structopt(alias = "stats")]
    Statistics(CommonOpts),

    #[structopt(alias = "roi")]
    ReturnOnInvestment {
        #[structopt(flatten)]
        options: CommonOpts,

        /// Query that returns the cash flows for the investment
        #[structopt(long = "--cash-flows")]
        cash_flows: Vec<String>,
        /// Query that returns the asset value
        #[structopt(long = "--assets-value")]
        assets_value: Vec<String>,
    },
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
struct Repl {
    #[structopt(flatten)]
    options: CommonOpts,
}
/// Command line options
#[derive(Debug, StructOpt, Clone)]
pub struct CommonOpts {
    /// Input file
    #[structopt(name = "FILE", short = "f", long = "file", parse(from_os_str))]
    pub input_file: PathBuf,

    /// Ignore init file if it exists
    #[structopt(long = "--args-only")]
    args_only: bool,

    /// Init file
    #[structopt(long = "--init-file", parse(from_os_str))]
    init_file: Option<PathBuf>,

    /// Depth
    #[structopt(short = "d", long = "depth")]
    pub depth: Option<usize>,

    /// The pattern to look for
    #[structopt(multiple = true, takes_value = true)]
    pub query: Vec<String>,
    /// Use only real postings rather than real and virtual
    #[structopt(long = "--real")]
    pub real: bool,
    #[structopt(short = "b", long = "begin", parse(try_from_str = date_parser))]
    pub begin: Option<NaiveDate>,
    #[structopt(short = "e", long = "end", parse(try_from_str = date_parser))]
    pub end: Option<NaiveDate>,
    #[structopt(short = "p", long = "period")]
    period: Option<String>,
    #[structopt(long = "now", parse(try_from_str = date_parser))]
    now: Option<NaiveDate>,

    /// Ignore balance assertions
    #[structopt(long = "--no-balance-check")]
    pub no_balance_check: bool,

    /// Display the report in the selected currency
    #[structopt(long = "--exchange", short = "-X")]
    pub exchange: Option<String>,

    /// TODO Date format
    #[structopt(long = "--date-format")]
    date_format: Option<String>,

    #[structopt(long = "--force-color")]
    force_color: bool,
    /// TODO force pager
    #[structopt(long = "--force-pager")]
    force_pager: bool,

    /// TODO effective
    #[structopt(long = "--effective")]
    effective: bool,

    /// Accounts, tags or commodities not previously declared will cause warnings.
    #[structopt(long = "--strict")]
    pub strict: bool,

    /// Accounts, tags or commodities not previously declared will cause errors.
    #[structopt(long = "--pedantic")]
    pub pedantic: bool,

    /// TODO Unrealized gains
    #[structopt(long = "--unrealized-gains")]
    unrealized_gains: Option<String>,
    /// TODO Unrealized losses
    #[structopt(long = "--unrealized-losses")]
    unrealized_losses: Option<String>,

    /// Whether to collapse postings from the same account in the same transaction
    #[structopt(long = "--collapse")]
    pub collapse: bool,
}

impl CommonOpts {
    pub fn new() -> Self {
        CommonOpts {
            input_file: PathBuf::new(),
            args_only: false,
            init_file: None,
            depth: None,
            query: vec![],
            real: false,
            begin: None,
            end: None,
            period: None,
            now: None,
            no_balance_check: false,
            exchange: None,
            date_format: None,
            force_color: false,
            force_pager: false,
            effective: false,
            strict: false,
            pedantic: false,
            unrealized_gains: None,
            unrealized_losses: None,
            collapse: false,
        }
    }
}

/// Entry point for the command line app
const INIT_FILE_FLAG: &str = "--init-file";
const NO_INIT_FILE_FLAG: &str = "--args-only";
const LEDGER_PATHS_UNDER_DIR: &str = "~/.ledgerrc";
const LEDGER_PATHS: &str = ".ledgerrc";

fn init_paths(args: Vec<String>) -> Result<Vec<String>, ()> {
    let mut possible_paths: Vec<String> = Vec::new();
    let mut ignore_init = false;
    for i in 0..args.len() {
        if args[i] == NO_INIT_FILE_FLAG {
            ignore_init = true;
            break;
        } else if args[i] == INIT_FILE_FLAG {
            let file = Path::new(&args[i + 1]);
            if !file.exists() {
                eprintln!("Config file '{}' does not exist", args[i + 1]);
                return Err(());
            }
            possible_paths.push(args[i + 1].clone());
            continue;
        }
    }

    if !ignore_init {
        possible_paths.push(shellexpand::tilde(LEDGER_PATHS_UNDER_DIR).to_string());
        possible_paths.push(LEDGER_PATHS.to_string());

        Ok(possible_paths)
    } else {
        Ok(vec![])
    }
}

/// Entry point for the command line app
pub fn run_app(input_args: Vec<String>) -> Result<(), ()> {
    let mut config_file = None;
    let possible_paths = init_paths(input_args.clone())?;
    for path in possible_paths.iter() {
        let file = Path::new(path);
        if file.exists() {
            config_file = Some(file);
            break;
        }
    }
    let args = if let Some(file) = config_file {
        parse_config_file(file, &input_args)
    } else {
        input_args
    };

    match Opt::from_iter_safe(args.iter()) {
        Err(error) => match Repl::from_iter_safe(args.iter()) {
            Ok(opt) => {
                if opt.options.query.len() > 0 {
                    error.exit()
                } else {
                    println!("dinero-rs v{}", VERSION);

                    let start = Instant::now();
                    let mut ledger = Ledger::try_from(&opt.options).unwrap();
                    let duration = start.elapsed();
                    println!(
                        "Loaded ledger from {:?} in {:?}",
                        &opt.options.input_file, duration
                    );

                    // Start the REPL
                    let mut rl = rustyline::Editor::<()>::new();
                    loop {
                        let readline = rl.readline(">> ");
                        match readline {
                            Ok(line) => match line.as_str() {
                                "exit" | "quit" => break,
                                "reload" => {
                                    let start = Instant::now();
                                    let journal = Ledger::try_from(&opt.options);
                                    let duration = start.elapsed();
                                    match journal {
                                        Ok(j) => {
                                            println!(
                                                "Loaded journal from {:?} in {:?}",
                                                &opt.options.input_file, duration
                                            );
                                            ledger = j;
                                        }
                                        Err(x) => {
                                            eprintln!("Journal could not be reloaded. Please check the errors and try again.");
                                            eprintln!("{}", x);
                                        }
                                    }
                                }
                                line => match line.trim().is_empty() {
                                    true => (),
                                    false => {
                                        let mut arguments: Vec<String> = Shlex::new(line).collect();
                                        if !line.starts_with("dinero ") {
                                            arguments.insert(0, String::from(""))
                                        }
                                        let args = if let Some(file) = config_file {
                                            parse_config_file(file, &arguments)
                                        } else {
                                            arguments
                                        };
                                        match Opt::from_iter_safe(args) {
                                            Ok(opt) => {
                                                match execute_command(opt, Some(ledger.clone())) {
                                                    Ok(_) => (),
                                                    Err(x) => eprintln!("{}\nThe above command resulted in an error. {:?}", line,x)
                                                }
                                            }
                                            Err(error) => {
                                                eprintln!("{}", error);
                                            }
                                        }
                                    }
                                },
                            },
                            Err(_) => break,
                        }
                    }
                }
                Ok(())
            }
            Err(_) => error.exit(),
        },
        Ok(opt) => execute_command(opt, None),
    }
}

fn parse_config_file(file: &Path, input_args: &Vec<String>) -> Vec<String> {
    let mut args = input_args.clone();

    let mut aliases = HashMap::new();
    aliases.insert("-f".to_string(), "--file".to_string());
    let contents = read_to_string(file).unwrap();
    for line in contents.lines() {
        let option = line.trim_start();
        match option.chars().nth(0) {
            Some(c) => match c {
                '-' => {
                    let message = format!("Bad config file {:?}\n{}", file, line);
                    assert!(line.starts_with("--"), "{}", message);
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
    args
}
fn execute_command(opt: Opt, maybe_ledger: Option<Ledger>) -> Result<(), ()> {
    // Print options
    if let Err(e) = match opt.cmd {
        Command::Balance {
            options,
            flat,
            no_total,
        } => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }
            balance::execute(&options, maybe_ledger, flat, !no_total)
        }
        Command::Register(options) => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }
            register::execute(&options, maybe_ledger)
        }

        Command::ReturnOnInvestment {
            options,
            cash_flows,
            assets_value,
        } => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }
            roi::execute(&options, maybe_ledger, cash_flows, assets_value)
        }
        Command::Commodities(options) => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }

            commodities::execute(&options, maybe_ledger)
        }
        Command::Payees(options) => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }

            payees::execute(&options, maybe_ledger)
        }
        Command::Prices(options) => prices::execute(&options, maybe_ledger),
        Command::Accounts(options) => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }

            accounts::execute(&options, maybe_ledger)
        }
        Command::Statistics(options) => {
            if options.force_color {
                env::set_var("CLICOLOR_FORCE", "1");
            }
            statistics::execute(&options, maybe_ledger)
        }
    } {
        let err_str = format!("{}", e);
        if err_str.len() > 0 {
            eprintln!("{}", err_str);
        }
        return Err(());
    }
    Ok(())
}

/// A parser for date expressions
pub fn date_parser(date: &str) -> Result<NaiveDate, Error> {
    lazy_static! {
        static ref RE_MONTH: Regex = Regex::new(r"(\d{4})[/-](\d\d?)$").unwrap();
        static ref RE_DATE: Regex = Regex::new(r"(\d{4})[/-](\d\d?)[/-](\d\d?)$").unwrap();
    }
    if RE_DATE.is_match(date) {
        let captures = RE_DATE.captures(date).unwrap();
        Ok(NaiveDate::from_ymd(
            captures.get(1).unwrap().as_str().parse::<i32>().unwrap(),
            captures.get(2).unwrap().as_str().parse::<u32>().unwrap(),
            captures.get(3).unwrap().as_str().parse::<u32>().unwrap(),
        ))
    } else if RE_MONTH.is_match(date) {
        let captures = RE_MONTH.captures(date).unwrap();
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
        assert_eq!(
            date_parser("2020-09-05").unwrap(),
            NaiveDate::from_ymd(2020, 9, 5)
        );
        assert_eq!(
            date_parser("2017-12-05").unwrap(),
            NaiveDate::from_ymd(2017, 12, 5)
        );
        assert_eq!(
            date_parser("2020-01-12").unwrap(),
            NaiveDate::from_ymd(2020, 1, 12)
        );
        assert_eq!(
            date_parser("2010-09").unwrap(),
            NaiveDate::from_ymd(2010, 9, 1)
        );
        // This test panics correctly, but it should be written elsewhere
        // assert!(date_parser("2020-13-12").is_err());
        assert!(date_parser("this is not a date").is_err());
    }

    #[test]
    fn test_balance() {
        let args: Vec<String> = vec![
            "testing",
            "bal",
            "-f",
            "tests/example_files/demo.ledger",
            "--init-file",
            "tests/example_files/example_ledgerrc",
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
        expected = "Bad config file \"tests/example_files/example_bad_ledgerrc\"\nThis line should be a comment but isn\'t, it is bad on purpose."
    )]
    fn bad_ledgerrc() {
        let args: Vec<String> = vec![
            "testing",
            "bal",
            "--init-file",
            "tests/example_files/example_bad_ledgerrc",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        let _res = run_app(args);
    }
    #[test]
    #[should_panic(
        expected = "Bad config file \"tests/example_files/example_bad_ledgerrc2\"\n- This does not parse either. And it shouldn't."
    )]
    fn other_bad_ledgerrc() {
        let args: Vec<String> = vec![
            "testing",
            "bal",
            "--init-file",
            "tests/example_files/example_bad_ledgerrc2",
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
