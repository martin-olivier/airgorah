<h1 align="center">
  <img src="icons/app_icon.png" width=100 height=100/><br>
Airgorah</h1>

<p align="center">
  <span>A WiFi auditing software that can perform deauth attacks and passwords cracking</span>
</p>

<p align="center">
  <a href="https://github.com/martin-olivier/airgorah/wiki/Installation">Installation</a>
  &nbsp;&nbsp;&nbsp;|&nbsp;&nbsp;&nbsp;
  <a href="https://github.com/martin-olivier/airgorah/wiki/Usage">Usage</a>
  &nbsp;&nbsp;&nbsp;|&nbsp;&nbsp;&nbsp;
  <a href="#contributing">Contributing</a>
</p>

![illustration](.github/assets/illustration.png)

[![crates](https://img.shields.io/crates/v/airgorah.svg)](https://crates.io/crates/airgorah)
[![aur](https://img.shields.io/aur/version/airgorah)](https://aur.archlinux.org/packages/airgorah)
[![ci](https://github.com/martin-olivier/airgorah/actions/workflows/CI.yml/badge.svg)](https://github.com/martin-olivier/airgorah/actions/workflows/CI.yml)

`Airgorah` is a WiFi auditing software that can discover the clients connected to an access point, perform deauthentication attacks against specific clients or all the clients connected to it, capture WPA handshakes, and crack the password of the access point.

It is written in Rust and uses [GTK4](https://github.com/gtk-rs/gtk4-rs) for the graphical part. The software is mainly based on [aircrack-ng](https://github.com/aircrack-ng/aircrack-ng) tools suite.

`⭐ Don't forget to put a star if you like the project!`

## Legal

⚠️ Airgorah is designed to be used in testing and discovering flaws in networks you are owner of. Performing attacks on WiFi networks you are not owner of is illegal in almost all countries. I am not responsible for whatever damage you may cause by using this software.

## Requirements

This software only works on `linux` and requires `root` privileges to run.

You will also need a wireless network card that supports `monitor mode` and `packet injection`.

## Installation

The installation instructions are available [here](https://github.com/martin-olivier/airgorah/wiki/Installation).

## Usage

The documentation about the usage of the application is available [here](https://github.com/martin-olivier/airgorah/wiki/Usage).

## License

This project is released under [MIT](LICENSE) license.

## Contributing

If you have any question about the usage of the application, do not hesitate to open a [discussion](https://github.com/martin-olivier/airgorah/discussions)

If you want to report a bug or provide a feature, do not hesitate to open an [issue](https://github.com/martin-olivier/airgorah/issues) or submit a [pull request](https://github.com/martin-olivier/airgorah/pulls)
