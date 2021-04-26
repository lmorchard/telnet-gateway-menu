# telnet-gateway-menu

A simple address book service for telnet hosts.

## What is this?

![20210423_220907](https://user-images.githubusercontent.com/21687/116024629-7c922f80-a603-11eb-8b9e-00dc978eebdd.jpg)

I have several old computers in my office. Each of them has a peripheral that connects to wifi and pretends to be a modem. But, instead of dialing a telephone number, I "dial" an internet address and the device connects via telnet.

This is fun, but I'm too lazy to set up an address book in the terminal software for each individual computer. So, I had the idea to connect to a service on my LAN that would present a menu of my favorite BBSes and connect from there. That way, every computer sees the same menu.

I also wanted an excuse to play with Rust some more.

## To do

* Read address book from disk

* Require return before accepting command input

* Better [actual telnet protocol support](https://github.com/envis10n/libtelnet-rs) - this is all completely naive so far

* Tests & CI

* GitHub Actions to generate cross-platform builds & releases

* Edit the address book from within the menus

* Log metadata like last-connection timestamp

* X/Y/ZMODEM transfers to/from the gateway computer for file sharing with the retro computers

* ZMODEM to download a URL

* Macros?

* Share one remote connection between multiple local computers simultaneously? (i.e. like screen?)

* Maybe somehow combine the functions of [tcpser] for a one-stop utility?

[tcpser]: https://github.com/go4retro/tcpser
