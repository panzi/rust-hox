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

* [ ] search for ASCII text
* [ ] search for binary (hex input)
* [ ] search for numbers
* [x] save selection to file
* [ ] display errors invlolving the above
* [x] highlight same as selection
* [ ] choose license (probably GPLv3)
