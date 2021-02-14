Hox
===

A very simple ncurses based hex viewer (read-only) written in Rust.

Why? Because all hex editors I've tried didn't work. At least not for files
larger than 2 GB.

Hex editors I've tried and how they've failed:

* Okteta: Can't open files > 2 GB
* Bless: Can't decode 64 bit integers
* ImHex binary: Linked against some random libraries that are not on my system.
  Why is it not statically linked/shipped with those libraries?
* ImHex compiled from source: Just crashes with "bad any_cast".
* ghex: Loads the full file into memory! And then you can't scroll to the
  bottom, probably i32 overflow in the scroll logic (jumps back up to start
  when scrolling down to about 2 GB).

General grieves with most hex GUI editors: They put offset and selection range
in the status bar where you can't copy the value. Why? You can copy all visible
text in an ncurses program.

Also in large files at least with Bless the status text is truncated and you
don't know where in the file you are.

Features
--------

* resizes to window
* decode bytes at curser as 8, 16, 32, and 64 bit integers, you can choose
  signed/unsinged, little/big endian
* jump to user supplied offset
* support large files via mmap()

TODO
----

* [x] search
  * [x] input for ASCII/UTF-8 text
  * [x] input for binary (hex input)
  * [x] input for integers
  * [x] actual search (forward/backward)
* [x] save selection to file
* [x] display errors invlolving the above
* [x] highlight same as selection
* [x] help pop-up
* [x] choose license (probably GPLv3)

GPLv3 License
-------------

[rust-hox](https://github.com/panzi/rust-hox) is free software: you can
redistribute it and/or modify it under the terms of the GNU General Public
License as published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

rust-hox is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE.  See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
rust-hox.  If not, see <https://www.gnu.org/licenses/>.
