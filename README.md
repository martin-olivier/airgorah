<h1 align="center">
  <img src="icons/app_icon.png" width=100 height=100/><br>
  Airgorah
  </a>
</h1>

<h4 align="center">A WiFi auditing software that can perform deauth attacks and passwords cracking</h4>

<p align="center">
  <a href="https://github.com/martin-olivier/airgorah/releases/tag/v0.4.2">
    <img src="https://img.shields.io/badge/Release-0.4.2-blue.svg" alt="version"/>
  </a>
  <a href="https://github.com/martin-olivier/airgorah/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-darkgreen.svg" alt="license"/>
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Language-Rust-orange.svg" alt="lang"/>
  </a>
  <a href="https://github.com/martin-olivier/airgorah/actions/workflows/CI.yml">
    <img src="https://github.com/martin-olivier/airgorah/actions/workflows/CI.yml/badge.svg" alt="ci"/>
  </a>
</p>

<p align="center">
  <a href="https://crates.io/crates/airgorah">
    <img src="https://img.shields.io/crates/v/airgorah.svg" alt="crates"/>
  </a>
  <a href="https://aur.archlinux.org/packages/airgorah">
    <img src="https://img.shields.io/aur/version/airgorah" alt="aur"/>
  </a>
</p>

![illustration](.github/assets/illustration.png)

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

## Documentation

The documentation about the usage of the application is available [here](https://github.com/martin-olivier/airgorah/wiki/Usage).

## License

This project is released under [MIT](LICENSE) license.

## Community

If you have any question about the usage of the application, do not hesitate to open a [discussion](https://github.com/martin-olivier/airgorah/discussions)

If you want to report a bug or provide a feature, do not hesitate to open an [issue](https://github.com/martin-olivier/airgorah/issues) or submit a [pull request](https://github.com/martin-olivier/airgorah/pulls)
