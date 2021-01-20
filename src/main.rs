use std::path::PathBuf;

use structopt::StructOpt;

use dinero::commands::{accounts, balance, check, commodities};

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
    // Prices,
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

    /// Depth
    #[structopt(short = "d", long = "depth")]
    depth: Option<usize>,

    /// The pattern to look for
    pattern: Option<String>,
}

fn main() {
    let opt: Opt = Opt::from_args();
    // println!("{:?}", opt);

    if let Err(e) = match opt.cmd {
        Command::Balance {
            options,
            flat,
            no_total,
        } => balance::execute(options.input_file, flat, !no_total, options.depth),
        Command::Accounts(options) => accounts::execute(options.input_file),
        Command::Commodities(options) => commodities::execute(options.input_file),
        Command::Check { input } => check::execute(input),
    } {
        eprintln!("{}", e);
    }
}
