use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let matches = App::new("dugon")
        .version("1.0")
        .author("Takayuki Maeda <takoyaki0316@gmail.com>")
        .about("A modern alternative to ‘PDFtk Server’.")
        .arg(
            Arg::with_name("path")
                .help("Sets an PDF file")
                .index(1)
                .required(true),
        )
        .get_matches();

    let mut file = File::open(matches.value_of("path").unwrap()).unwrap();
}
