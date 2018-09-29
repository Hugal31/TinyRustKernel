#!/usr/bin/env python3

import sys
from PIL import Image

image = Image.open(sys.argv[1])
rgb_image = image.convert('RGB')

with open('out.vga', 'wb') as out:
    for y in range(200):
        for x in range(320):
            r, g, b = rgb_image.getpixel((x, y))
            if r == 255 and g == b == 0:
                out.write(bytes([0x4]))
            elif r == g == b and r >= 127:
                out.write(bytes([0x80]))
            elif r == g == b == 0:
                out.write(bytes([0x0]))
            else:
                print(r, g, b)
                out.write(bytes([0x30]))
