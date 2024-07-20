# tRust_ip

This application can be used as middleware for reverse proxies like Traefik. This application checks the X-FORWARDED-FOR header against a static list of IP's and optionally, the Atlassian ip range (https://ip-ranges.atlassian.com).

## Environment variables

One or both of the below environment variables should be present, otherwise, every request will be denied.

### WHITELIST
Takes a comma separated string (no spaces) as a list of IP's to allow access to

### ATLASSIAN_IP_URL

Queries this URL and parses its JSON reply for CIDR blocks. Then it checks the X_FORWARDED_FOR ip address against it.

## Examples

See the docker-compose for an example.