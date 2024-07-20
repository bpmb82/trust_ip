#!/bin/bash

echo "** This is an unauthorized IP:"
curl -v -k --header "X-Forwarded-For: 192.168.0.2" localhost:8080/

echo ""
echo ""
echo "** This is an authorized IP:"

curl -v --header "X-Forwarded-For: 192.168.1.79" localhost:8080/
echo ""
echo ""
echo "** This is an authorized Atlassian IP:"

curl -v --header "X-Forwarded-For: 3.26.128.129" localhost:8080/
echo ""
