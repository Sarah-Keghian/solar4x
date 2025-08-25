# Quickstart Guide

## Building the game from source
1. Install the latest stable version of [Rust](https://www.rust-lang.org/).
2. Install the OS dependencies for Bevy following their [instructions](https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies).
3. Fork the GitHub repository.
4. Clone your new repository and navigate into it:
   ```bash
   git clone <repository-url>
   cd <repository-directory>

5. Inside the project folder, build the game (compiling Bevy takes time):
    ```bash
    cargo build
6. Run the game:
    ```bash
    cargo run --bin client

Optionnally, you can clone the repository first and use the provided Nix shell to install Rust and the Bevy OS dependencies automatically using [Nix](https://nixos.org/) magic!

## Adding a new feature

To add a new feature, follow these steps:

1. Make sure you are on the `dev` branch:
   ```bash
   git checkout dev

2. Create a new branch for your feature:
    ```bash
   git checkout -b feature/name-of-the-feature
3. Work on your feature in this branch
4. When your feature is ready, you can push the branch to your fork