#!/bin/bash

openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem -days 3650 -config ssl.conf -nodes -sha256 -extensions 'req_ext'
