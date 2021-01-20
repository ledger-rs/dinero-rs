use structopt::StructOpt;
use dinero::commands::{check, balance, accounts, commodities};
use std::path::PathBuf;
use dinero::Error;

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
    input: PathBuf,

    /// The pattern to look for
    pattern: Option<String>,
}

fn main() {
    let opt: Opt = Opt::from_args();
    // println!("{:?}", opt);

    if let Err(e) = match opt.cmd {
        Command::Balance { options, flat, no_total } => {
            balance::execute(options.input, flat, !no_total)
        }
        Command::Accounts(options) => { accounts::execute(options.input) }
        Command::Commodities(options) => { commodities::execute(options.input) }
        Command::Check { input } => { check::execute(input) }
    } {
        eprintln!("{}", e);
    }
}