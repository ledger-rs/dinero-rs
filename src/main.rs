fn main() {
    println!("There is a lot of work to do...");
}
/*
use std::env;
use std::path::Path;

mod lib;

#[derive(Debug)]
enum Command {
    Print,
    Balance,
    Register,
    Accounts,
    Codes,
    Payees,
    Prices,
    Commodities,
}

#[derive(PartialEq)]
enum Argument {
    Flat,
    Tree,
    Raw,
    Explicit,
}

fn main() {
    if let Err(e) = start() {
        eprintln!("{}", e);
    }
}

fn start() -> Result<(), String> {
    let mut files: Vec<String> = Vec::new();
    let mut command = None;
    let mut arguments = Vec::new();
    let mut items = Vec::new();

    parse_arguments(&mut files, &mut command, &mut arguments)?;

    match command {
        None => Err(String::from("Error : No command selected")),
        Some(command) => {
            if files.is_empty() {
                return Err(String::from(
                    "Error : No file(s) selected. Try --file <file> to select a file",
                ));
            }
            let paths = files.iter().map(|f| Path::new(f)).collect();
            parsers::parse_files(paths)?;

            // TODO this will not work
            execute_command(command, arguments, items)
        }
    }
}

fn parse_arguments(
    files: &mut Vec<String>,
    command: &mut Option<Command>,
    arguments: &mut Vec<Argument>,
) -> Result<(), String> {
    let mut it = env::args().skip(1);

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--file" | "-f" => match it.next() {
                None => return Err(String::from("Error : No argument provided for --file")),
                Some(file_path) => files.push(file_path),
            },
            "print" => *command = Some(Command::Print),
            "accounts" => *command = Some(Command::Accounts),
            _ => {}
        }
    }

    Ok(())
}

fn execute_command(
    command: Command,
    arguments: Vec<Argument>,
    items: Vec<model::Item>,
) -> Result<(), String> {
    match command {
        Command::Print => {
            if arguments.contains(&Argument::Explicit) {
                return commands::print::print_explicit(items);
            }
            if arguments.contains(&Argument::Raw) {
                return commands::print::print_raw(items);
            }
            return commands::print::print_raw(items);
        }
        Command::Codes => commands::codes::print(items)?,
        _ => return Err(
            format!("Command {:?} not implemented", command)
        )
    }
    Ok(())
}
*/