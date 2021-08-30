use std::env;

fn main() {
    match dinero::run_app(env::args().collect()) {
        Ok(_) => std::process::exit(0),
        Err(x) => {
            let message = format!("{}", x);
            if !message.is_empty() {
                eprintln!("{}", message);
            }
            std::process::exit(1)
        }
    }
}
