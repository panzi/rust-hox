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
* decode bytes at curser as 8, 16, 32, and 64 bit integers, 32 and 64 bit
  floating point numbers, you can choose signed/unsinged, little/big endian
* jump to user supplied offset
* support large files via mmap()

Help
----

```plain
Hotkeys
═══════
h or F1 ... show this help message
q ......... quit
e ......... toggle between big and little endian
i ......... toggle between signed and unsinged
o ......... enter offset to jump to
s ......... toggle select mode
S ......... clear selection
w ......... write selection to file
f or F3 ... open search bar (and search for current selection)
F ......... clear search
n ......... find next
p ......... find previous
# ......... select ASCII line under cursor

Search
──────
Enter or F3 ... find (next)
F5 ............ switch through modes: Text/Binary/Integer
Shift+F5 ...... switch through modes in reverse
F6 ............ switch through integer sizes: 8/16/32/64
F7 ............ toggle signed/unsigned
F8 ............ toggle little endian/big endian
Escape ........ stop search

Non-Text Search
───────────────
Escape or q ... stop search
(and all other global hotkeys)

Navigation
──────────
← ↑ ↓ → ..... move cursor
Home ........ move cursor to start of line
End ......... move cursor to end of line
Ctr+Home .... move cursor to start of file
Ctr+End ..... move cursor to end of file
Page Up ..... move view up one page
Page Down ... move view down one page

Press Enter, Escape or any normal key to clear errors.
```

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
