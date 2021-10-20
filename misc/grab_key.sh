#!/bin/bash

xxd -r text.log > gptr.bin
./xor.py gptr.bin 0x20
dd if=gptr.bin of=key.bin bs=4096 count=1 skip=1
