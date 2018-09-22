#!/usr/bin/env python3

import sys

with open(sys.argv[1] + '.txt') as ifile, open(sys.argv[1] + '.vga', 'wb') as ofile:
    for line in ifile.readlines():
        for c in line[:-1]:
            if c in ' B_+&%#':
                ofile.write(b'\x00\x00')
            elif c == '$':
                ofile.write(b'\x00\xF0')
            elif c in ',:*':
                ofile.write(b'\x00\x60')
            else:
                ofile.write(b'\x00\x40')
