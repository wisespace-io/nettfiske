[![Build Status](https://travis-ci.org/wisespace-io/nettfisk.png?branch=master)](https://travis-ci.org/wisespace-io/nettfisk)
[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE-MIT)
[![Apache-2.0 licensed](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE-APACHE)

# Nettfisk

Uses [certstream](https://certstream.calidog.io/) SSL certificates live stream to identify possible phishing domain names. It is inspired by Phishing Catcher (https://github.com/x0rz/phishing_catcher). (WIP)

## Usage

cargo run --release

### Use Cases

Attempt to detect the use of Punycode and Homoglyph Attacks to obfuscate Domains. The homograph protection mechanism in Chrome, Firefox, and Opera may fail when some characters are replaced with a similar character from a foreign language.

Example: microsoft.com⁄index.html.irongeek.com
         microsoft.xn--comindex-g03d.html.irongeek.com

The slash symbol in the first url is not really a slash symbol at all. Also adding a SSL certificate can take few minutes and the user can feel safer with the locker next to domain.

Example, try to open the domain https://www.xn--80ak6aa92e.com/ on Firefox.