use std::env;

fn main() {
    match dinero::run_application(env::args().collect()) {
        Ok(_) => {}
        Err(_) => std::process::exit(1),
    }
}
