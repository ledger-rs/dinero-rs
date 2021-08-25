use std::env;

fn main() {
    match dinero::run_app(env::args().collect()) {
        Ok(_) => {}
        Err(x) => {
            eprintln!("{}", x);
            std::process::exit(1)
        },
    }
}
