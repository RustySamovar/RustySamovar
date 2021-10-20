#!/bin/bash

echo -ne "\x45\x63\x32\x62" >  fake.ec2b
echo -ne "\x10\x00\x00\x00" >> fake.ec2b
dd if=/dev/urandom bs=16 count=1 >> fake.ec2b
echo -ne "\x00\x08\x00\x00" >> fake.ec2b
dd if=/dev/urandom bs=2048 count=1 >> fake.ec2b
