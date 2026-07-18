#[derive(clap::Parser)]
#[command(version, about)]
pub struct Args {
    pub source: String,
}
