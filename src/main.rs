use clap::{App, Arg};

fn main() {
    let _matches = App::new("dugon")
        .version("1.0")
        .author("Takayuki Maeda <takoyaki0316@gmail.com>")
        .about("A modern alternative to ‘PDFtk Server’.")
        .arg(Arg::with_name("path").help("Sets an PDF file"))
        .get_matches();
}
