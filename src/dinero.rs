use std::env;

fn main() {
    match dinero::run_app(env::args().collect()) {
        Ok(_) => std::process::exit(0),
        Err(_) => std::process::exit(1),
    }
}
