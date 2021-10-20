#!/usr/bin/env python3

import sys

filename = sys.argv[1]
key = int(sys.argv[2], 16)

data = open(filename, "rb").read()
open(filename, "wb").write(bytes([i ^ key for i in data]))
