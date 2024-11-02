#!/bin/bash

echo "** This is an unauthorized IP (192.168.0.2):"
curl -v -k --header "X-Forwarded-For: 192.168.0.2" localhost:8080/

echo ""
echo ""
echo "** This is an authorized IP (127.0.0.1):"

curl -v --header "X-Forwarded-For: 127.0.0.1" localhost:8080/
echo ""
echo ""
echo "** This is an authorized Atlassian IP (3.26.128.129):"

curl -v --header "X-Forwarded-For: 3.26.128.129" localhost:8080/
echo ""
