use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dinero::run_app(env::args().collect())
}
