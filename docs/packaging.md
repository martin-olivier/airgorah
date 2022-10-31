# Create a package and install it

First, you will need to clone the project using the following command:

```sh
git clone https://github.com/martin-olivier/airgorah.git
cd airgorah
```

Then, to create a `debian` package, you can use the following command:

```sh
cargo install cargo-deb
cargo deb
```

After those commands, a `.deb` package will be created in the `target/debian` folder. You can install it with the following command:

```sh
sudo apt install <path_to_.deb>
```
