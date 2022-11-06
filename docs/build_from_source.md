# Build from source

## 1. Install Cargo

To build the project, you will need to install [cargo](https://www.rust-lang.org/tools/install), the rust compiler:

```sh
curl https://sh.rustup.rs -sSf | sh
```

## 2. Install Dependencies

Then, you will need to install `airgorah` dependencies:

### Debian (Ubuntu, PopOS, Mint, Kali)

```sh
sudo apt install dbus-x11 libgtk-4-dev libglib2.0-dev aircrack-ng wireless-tools
```

### Fedora

```sh
sudo dnf install dbus-x11 gtk4-devel glib2-devel aircrack-ng wireless-tools-1
```

## 3. Install Airgorah

After those steps, you will be able to:

- [Create a package and install it](packaging.md) (recommended)
- [Build and install from crates.io](installation.md)
