<h1 align="center">
  <img src="icons/app_icon.png" width=100 height=100/><br>
  Airgorah
  </a>
</h1>

<h4 align="center">A WiFi auditing software that can performs deauth attacks and WPA passwords recovery</h4>

<p align="center">
  <a href="https://github.com/martin-olivier/airgorah/releases/tag/v0.1.0">
    <img src="https://img.shields.io/badge/Version-0.1.0_(beta)-blue.svg" alt="version"/>
  </a>
  <a href="https://github.com/martin-olivier/airgorah/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-darkgreen.svg" alt="license"/>
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Language-Rust-orange.svg" alt="cppversion"/>
  </a>
  <a href="https://github.com/martin-olivier/airgorah/actions/workflows/CI.yml">
    <img src="https://github.com/martin-olivier/airgorah/actions/workflows/CI.yml/badge.svg" alt="ci"/>
  </a>
</p>

Airgorah is written in Rust and uses [GTK4 bindings](https://github.com/gtk-rs/gtk4-rs) for the graphical part. The software is based on [aircrack-ng](https://github.com/aircrack-ng/aircrack-ng) tools suite.

⚠️ This software is for educational purposes only. It should not be used for illegal activity. The author is not responsible for its use.

![illustration](.github/assets/illustration.png)

`⭐ Don't forget to put a star if you like the project!`

## Requirements

This software only works on `linux` and requires `root` privileges to run.

## Installation

You can find pre-built releases for Debian [here](https://github.com/martin-olivier/airgorah/releases/latest).

You will just have to download the debian package and install it with the following command:

```sh
sudo apt install <path_to_deb_package>
```

Otherwise, if you'd like to compile from source, you can follow this [guide](docs/build_from_source.md).

## Future features

- [ ] WPA handshake capture
- [ ] WPA handshake decryption (dictionary / bruteforce)
- [ ] WPS attack feature
- [ ] WEP attack feature

- [ ] Provide releases for other linux distributions (Arch, Fedora, ...)
- [ ] Improve the code quality and the documentation
- [ ] Improve the UI
