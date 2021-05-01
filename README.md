# telnet-gateway-menu

[![github actions badge](https://github.com/lmorchard/telnet-gateway-menu/actions/workflows/test.yml/badge.svg)](https://github.com/lmorchard/telnet-gateway-menu/actions)

A simple address book service for telnet hosts.

## What is this?

![Capture](https://user-images.githubusercontent.com/21687/116037482-069ac200-a61d-11eb-8c7a-f67b53983149.PNG)

![20210423_220907](https://user-images.githubusercontent.com/21687/116024629-7c922f80-a603-11eb-8b9e-00dc978eebdd.jpg)

I have several old computers in my office. Each of them has a peripheral that connects to wifi and pretends to be a modem. But, instead of dialing a telephone number, I "dial" an internet address and the device connects via telnet.

This is fun, but I'm too lazy to set up an address book in the terminal software for each individual computer. So, I had the idea to connect to a service on my LAN that would present a menu of my favorite BBSes and connect from there. That way, every computer sees the same menu.

I also wanted an excuse to play with Rust some more.

## To do

* Package up some doco with the executable for release?

* Better [actual telnet protocol support](https://github.com/envis10n/libtelnet-rs) - this is all completely naive so far

* Edit the address book from within the menus

* Log metadata like last-connection timestamp

* X/Y/ZMODEM transfers to/from the gateway computer for file sharing with the retro computers

* ZMODEM to download a URL

* Username / password auto-fill?

* Macros?

* Share one remote connection between multiple local computers simultaneously? (i.e. like screen?)

* Maybe somehow combine the functions of [tcpser] for a one-stop utility?

* Build a Docker image for funsies?

* `cargo publish` from github? (maybe not for many versions, this thing is not worth cluttering up crates.io yet)

[tcpser]: https://github.com/go4retro/tcpser
