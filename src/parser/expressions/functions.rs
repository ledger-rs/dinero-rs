use colored::Colorize;

use crate::CommonOpts;

use super::EvalResult;
pub(super) fn concatenate(args: Vec<EvalResult>) -> EvalResult {
    let string = args
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("");
    EvalResult::String(string)
}

pub(super) fn format_date(args: Vec<EvalResult>, options: &CommonOpts) -> EvalResult {
    let date = match &args[0] {
        EvalResult::Date(d) => d,
        x => panic!("Should be a date, found {:?}", x),
    };
    let string = match args.len() {
        1 => {
            format!("{}", date)
        }
        2 => date.format(args[1].to_string().as_str()).to_string(),
        x => panic!("Expected one or two arguments, not {}", x),
    };
    EvalResult::String(string)
}

pub(super) fn int(arg: &EvalResult) -> EvalResult {
    EvalResult::Usize(usize::from_str_radix(arg.to_string().as_str(), 10).unwrap())
}
pub(super) fn not(arg: &EvalResult) -> EvalResult {
    if let EvalResult::Boolean(b) = arg {
        EvalResult::Boolean(!*b)
    } else {
        panic!("Expected boolean, found {:?}", arg)
    }
}

/// justify value first_width latter_width right_justify colorize
///
/// Right or left justify the string representing value. The width of the field in the first line is given by first_width. For subsequent lines the width is given by latter_width. If latter_width=-1, then first_width is used for all lines. If right_justify=true then the field is right justified within the width of the field. If it is false, then the field is left justified and padded to the full width of the field. If colorize is true, then ledger will honor color settings.

pub(super) fn justify(args: Vec<EvalResult>, options: &CommonOpts) -> EvalResult {
    let raw = args[0].to_string();
    let first_width = if let EvalResult::Usize(x) = args[1] {
        x
    } else {
        30
    };
    let latter_width = if args.len() > 2 {
        if let EvalResult::Usize(x) = args[2] {
            x
        } else {
            first_width
        }
    } else {
        first_width
    };
    let right_justify = if args.len() > 3 {
        if let EvalResult::Boolean(x) = args[3] {
            x
        } else {
            false
        }
    } else {
        false
    };
    // todo colorize
    let colorize = if args.len() > 4 {
        if let EvalResult::Boolean(x) = args[4] {
            x
        } else {
            false
        }
    } else {
        false
    };

    let formatted = match right_justify {
        false => format!("{:<width$}", raw, width = first_width),
        true => format!("{:>width$}", raw, width = first_width),
    };
    EvalResult::String(formatted)
}

/// scrub
pub(super) fn scrub(value: &EvalResult, options: &CommonOpts) -> EvalResult {
    EvalResult::String(value.to_string())
}

/// if expression
pub(super) fn if_expression(val_if_true: EvalResult, condition: EvalResult) -> EvalResult {
    match condition {
        EvalResult::Boolean(x) => {
            if x {
                val_if_true
            } else {
                EvalResult::Result(Box::new(None))
            }
        }
        x => {
            eprintln!("Expected Boolean, found {:?}", x);

            EvalResult::Result(Box::new(None))
        }
    }
}

/// colored
// todo add styles
// todo shouldn't this be three arguments?
pub(super) fn ansify_if(string: &EvalResult, color_style: &EvalResult) -> EvalResult {
    if let &EvalResult::Color(c) = color_style {
        match c {
            colored::Color::Black => EvalResult::String(format!("{}", string.to_string().black())),
            colored::Color::Red => EvalResult::String(format!("{}", string.to_string().red())),
            colored::Color::Green => EvalResult::String(format!("{}", string.to_string().green())),
            colored::Color::Yellow => {
                EvalResult::String(format!("{}", string.to_string().yellow()))
            }
            colored::Color::Blue => EvalResult::String(format!("{}", string.to_string().blue())),
            colored::Color::Magenta => {
                EvalResult::String(format!("{}", string.to_string().magenta()))
            }
            colored::Color::Cyan => EvalResult::String(format!("{}", string.to_string().cyan())),
            colored::Color::White => EvalResult::String(format!("{}", string.to_string().white())),
            colored::Color::BrightBlack => {
                EvalResult::String(format!("{}", string.to_string().bright_black()))
            }
            colored::Color::BrightRed => {
                EvalResult::String(format!("{}", string.to_string().bright_red()))
            }
            colored::Color::BrightGreen => {
                EvalResult::String(format!("{}", string.to_string().bright_green()))
            }
            colored::Color::BrightYellow => {
                EvalResult::String(format!("{}", string.to_string().bright_yellow()))
            }
            colored::Color::BrightBlue => {
                EvalResult::String(format!("{}", string.to_string().bright_blue()))
            }
            colored::Color::BrightMagenta => {
                EvalResult::String(format!("{}", string.to_string().bright_magenta()))
            }
            colored::Color::BrightCyan => {
                EvalResult::String(format!("{}", string.to_string().bright_cyan()))
            }
            colored::Color::BrightWhite => {
                EvalResult::String(format!("{}", string.to_string().bright_white()))
            }
            colored::Color::TrueColor { r, g, b } => {
                EvalResult::String(format!("{}", string.to_string().truecolor(r, g, b)))
            }
        }
    } else if let &EvalResult::Style(s) = color_style {
        match s {
            colored::Styles::Clear => EvalResult::String(String::new()),
            colored::Styles::Bold => EvalResult::String(format!("{}", string.to_string().bold())),
            colored::Styles::Dimmed => {
                EvalResult::String(format!("{}", string.to_string().dimmed()))
            }
            colored::Styles::Underline => {
                EvalResult::String(format!("{}", string.to_string().underline()))
            }
            colored::Styles::Reversed => {
                EvalResult::String(format!("{}", string.to_string().reversed()))
            }
            colored::Styles::Italic => {
                EvalResult::String(format!("{}", string.to_string().italic()))
            }
            colored::Styles::Blink => EvalResult::String(format!("{}", string.to_string().blink())),
            colored::Styles::Hidden => {
                EvalResult::String(format!("{}", string.to_string().hidden()))
            }
            colored::Styles::Strikethrough => {
                EvalResult::String(format!("{}", string.to_string().strikethrough()))
            }
        }
    } else {
        EvalResult::String(string.to_string())
    }
}

/// truncated
// todo make it more sophisticated, especially for accounts
pub(super) fn truncated(string: &EvalResult, width: &EvalResult) -> EvalResult {
    let w: usize = if let EvalResult::Usize(x) = width {
        *x
    } else {
        panic!("Expected usize, found {:?}", width);
    };
    return EvalResult::String(string.to_string().chars().take(w).collect());
}
