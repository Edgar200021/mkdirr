use mkdirr::{read_config, run};

fn main() {
    if let Err(err) = read_config().and_then(|config| run(&config)) {
        eprintln!("{}", err);
    }
}
