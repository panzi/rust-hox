// This file is part of rust-hox.
//
// rust-hox is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rust-hox is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-hox.  If not, see <https://www.gnu.org/licenses/>.

use clap::{Arg, App};

mod result;
mod hox;
mod mmap;
mod input_widget;
mod number_input;
mod file_input;
mod text_box;
mod search_widget;
mod consts;

use result::Result;
use hox::{Hox, Endian, Theme};

fn main() {
    let args = App::new("Hox - Hex viewer written in Rust")
        .version("1.0.0")
        .author("Mathias Panzenböck <grosser.meister.morti@gmx.net>")

        .arg(Arg::with_name("endian")
            .long("endian")
            .short("e")
            .default_value("little")
            .takes_value(true)
            .help("Display numbers as 'little' or 'big' endian."))

        .arg(Arg::with_name("signed")
            .long("signed")
            .short("s")
            .takes_value(false)
            .help("Display numbers as signed."))

        .arg(Arg::with_name("dark-mode")
            .long("dark-mode")
            .takes_value(false)
            .conflicts_with("light-mode")
            .help("Plesant dark mode. [default]"))

        .arg(Arg::with_name("light-mode")
            .long("light-mode")
            .takes_value(false)
            .help("Burn your eyes in light mode."))

        .arg(Arg::with_name("file")
            .index(1)
            .required(true)
            .value_name("FILE"))
        .get_matches();

    let filename = args.value_of("file").unwrap();

    let endian = args.value_of("endian").unwrap();
    let endian = if endian.eq_ignore_ascii_case("little") {
        Endian::Little
    } else if endian.eq_ignore_ascii_case("big") {
        Endian::Big
    } else {
        eprintln!("Error: illegal value for --endian: {:?}", endian);
        std::process::exit(1);
    };

    let signed = args.is_present("signed");
    let theme = if args.is_present("light-mode") {
        Theme::Light
    } else {
        Theme::Dark
    };

    if let Err(mut error) = run(filename, endian, signed, theme) {
        if error.path().is_none() {
            error = error.with_path(filename);
        }
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}

fn run(filename: &str, endian: Endian, signed: bool, theme: Theme) -> Result<()> {
    let mut file = std::fs::File::open(filename)?;

    let mut hox = Hox::new(&mut file, theme)?;
    hox.set_endian(endian);
    hox.set_signed(signed);

    hox.run()
}
