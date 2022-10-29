<h1 align="center">
  Airgorah
</h1>
<p align="center">
  <a href="https://github.com/martin-olivier/airgorah/releases/tag/v0.1.0">
    <img src="https://img.shields.io/badge/Version-0.1.0-blue.svg" alt="version"/>
  </a>
  <a href="https://github.com/martin-olivier/airgorah/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-darkgreen.svg" alt="license"/>
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/Language-Rust-orange.svg" alt="cppversion"/>
  </a>
</p>

Airgorah is a WiFi auditing software that can performs deauth attacks and WPA passwords recovery

`⭐ Don't forget to put a star if you like the project!`

## ⚠️ Requirements
This software only works on `Linux` distributions (Ubuntu, Debian, Arch, etc.) and requires `root` privileges to run.

## 💻 Installation

### 1. Install Cargo

To build the project, you will need to install [cargo](https://www.rust-lang.org/tools/install), the rust compiler:

```sh
curl https://sh.rustup.rs -sSf | sh
```

### 2. Install Dependencies

Then, you will need to install `airgorah` build and runtime dependencies:

**APT**
```sh
sudo apt install dbus-x11 libgtk-4-dev aircrack-ng
```

**DNF**
```sh
sudo dnf install dbus-x11 gtk4-devel aircrack-ng
```

**PACMAN**
```sh
sudo pacman -S dbus gtk4 aircrack-ng
```

### 3. Install Airgorah

Then, you will be able to build and install `airgorah` on your computer:

```sh
git clone https://github.com/martin-olivier/airgorah -b feat/app
cargo install --path airgorah
```

## 🚀 Usage

To run the application, use the following command:

```sh
sudo airgorah
```
