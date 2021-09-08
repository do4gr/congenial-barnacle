use clap::{App, Arg};
use engine_lib::core_logic;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Transaction Engine")
        .version("0.1")
        .author("do4gr")
        .about("Processes transactions input as csv, outputs client account states to stdout.")
        .arg(
            Arg::new("INPUT")
                .about("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    if let Some(input_file_path) = matches.value_of("INPUT") {
        core_logic(input_file_path).await?
    }

    Ok(())
}
