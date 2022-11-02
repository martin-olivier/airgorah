# Create a package and install it

First, you will need to clone the project using the following command:

```sh
git clone https://github.com/martin-olivier/airgorah.git
cd airgorah
```

Then, you will be able to create a `debian` package by running the following commands:

```sh
cargo install cargo-deb
cargo deb
```

After those commands, a `.deb` package will be created in the `target/debian` folder. You can install it with the following command:

```sh
sudo apt install <path_to_deb_package>
```
