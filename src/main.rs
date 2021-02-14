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

use result::{Result, Error};
use hox::{Hox, Endian};

fn main() -> Result<()> {
    let args = App::new("Hox - Hex viewer written in Rust")
        .version("0.1.0")
        .author("Mathias Panzenböck <grosser.meister.morti@gmx.net>")
        
        // TODO: make this changeable at runtime
        .arg(Arg::with_name("endian")
            .long("endian")
            .short("e")
            .default_value("little")
            .takes_value(true)
            .help("Display numbers as 'little' or 'big' endian."))

        .arg(Arg::with_name("signed")
            .long("signed")
            .takes_value(false)
            .help("Display numbers as signed."))

        .arg(Arg::with_name("file")
            .index(1)
            .required(true)
            .value_name("FILE"))
        .get_matches();

    let filename = match args.value_of("file") {
        Some(filename) => filename,
        None => return Err(Error::message("argument FILE is required")),
    };

    let endian = args.value_of("endian").unwrap();

    let endian = if endian.eq_ignore_ascii_case("little") {
        Endian::Little
    } else if endian.eq_ignore_ascii_case("big") {
        Endian::Big
    } else {
        return Err(Error::message(format!("illegal value for --endian: {:?}", endian)));
    };

    let signed = args.is_present("signed");

    match run(filename, endian, signed) {
        Ok(()) => Ok(()),
        Err(error) => {
            if error.path().is_none() {
                return Err(error.with_path(filename));
            } else {
                return Err(error);
            }
        }
    }
}

fn run(filename: &str, endian: Endian, signed: bool) -> Result<()> {
    let mut file = std::fs::File::open(filename)?;

    let mut hox = Hox::new(&mut file)?;
    hox.set_endian(endian);
    hox.set_signed(signed);

    hox.run()
}
