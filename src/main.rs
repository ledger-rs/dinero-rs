use std::env;

fn main() {
    match dinero::run_app(env::args().collect()) {
        Ok(_) => {}
        Err(_) => std::process::exit(1),
    }
}
