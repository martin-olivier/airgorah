# Build from source

## 1. Install Cargo

To build the project, you will need to install [cargo](https://www.rust-lang.org/tools/install), the rust compiler:

```sh
curl https://sh.rustup.rs -sSf | sh
```

## 2. Install Dependencies

Then, you will need to install `airgorah` dependencies:

### Debian (Ubuntu, PopOS, Mint, Kali, etc.)

```sh
sudo apt install dbus-x11 libgtk-4-dev aircrack-ng
```

### Fedora

```sh
sudo dnf install dbus-x11 gtk4-devel aircrack-ng
```

## 3. Install Airgorah

After those steps, you can:

- [Create a package and install it](packaging.md) (recommended)
- [Build and install from crates.io](installation.md)
