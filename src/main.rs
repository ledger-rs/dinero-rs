use std::path::{PathBuf, Path};

use structopt::StructOpt;

use dinero::commands::{accounts, balance, check, commodities, prices};
use std::env;
use std::fs::read_to_string;
use std::collections::HashMap;

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
    // Register,
    /// List the accounts
    Accounts(CommonOpts),
    // Codes,
    // Payees,
    /// Show the exchange rates
    Prices(CommonOpts),
    /// List commodities
    #[structopt(alias = "currencies")]
    Commodities(CommonOpts),
    Check {
        #[structopt(name = "FILE", short = "f", long = "file", parse(from_os_str))]
        input: PathBuf,
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
struct CommonOpts {
    /// Input file
    #[structopt(name = "FILE", short = "f", long = "file", parse(from_os_str))]
    input_file: PathBuf,

    /// Init file
    #[structopt(long = "--init-file", parse(from_os_str))]
    init_file: Option<PathBuf>,

    /// Depth
    #[structopt(short = "d", long = "depth")]
    depth: Option<usize>,

    /// The pattern to look for
    pattern: Option<String>,

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

fn main() {
    let mut args: Vec<String> = env::args().collect();
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
        aliases.insert(
            "-f".to_string(),
            "--file".to_string(),
        );
        let contents = read_to_string(file).unwrap();
        for line in contents.lines() {
            let option = line.trim_start();
            match option.chars().nth(0) {
                Some(c) => match c {
                    '-' => {
                        assert!(line.starts_with("--"),
                                format!("Bad config file {:?}\n{}", file, line));
                        let mut iter = line.split_whitespace();
                        let option = iter.next().unwrap();
                        if !args.iter()
                            .any(|x| (x == option) |
                                (aliases.get(x).unwrap_or(&String::new()) == option)) {
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
                    ';' | '#' | '!' | '%' => (),  // a comment

                    _ => panic!("Bad config file {:?}\n{}", file, line),
                },
                None => (),
            }
        }
    }
    let opt: Opt = Opt::from_iter(args.iter());

    if let Err(e) = match opt.cmd {
        Command::Balance {
            options,
            flat,
            no_total,
        } => balance::execute(options.input_file, flat, !no_total, options.depth),
        Command::Commodities(options) => commodities::execute(options.input_file),
        Command::Prices(options) => prices::execute(options.input_file),
        Command::Accounts(options) => accounts::execute(options.input_file),
        Command::Check { input } => check::execute(input),
    } {
        eprintln!("{}", e);
    }
}
