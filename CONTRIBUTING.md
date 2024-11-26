# Guide to contributing
---

Thanks for reading! You are awesome!

## Prerequisites
- Rust
  - stable: Clippy is written for stable rust.
  - nightly: The only reason Nightly is being used is due to using [`rustfmt`](https://github.com/rust-lang/rustfmt)'s nightly release. This may change in the future though.
    ```bash
    rustup default stable
    rustup toolchain install nightly
    ```

- Cargo
  - rustfmt & clippy
    ```bash
    rustup component add rustfmt
    rustup component add rustfmt --toolchain nightly
    ```

## Code style
There is a script to perform all code styling per Clippy guidelines

> [!TIP]
> Before submitting a PR, run the check script using:
>
>     ./check.sh
>
> This will fail if you are do not have nightly rust and `rustfmt` installed! See [Prerequsites](#Prerequsites)

## xtask
Clippy uses [xtask](https://github.com/matklad/cargo-xtask/) to generate man pages and shell
completions before release.

> [!NOTE]
> Generating these are easy.
> Before making a PR that will change:
> - **man pages**: `cargo gen man`
> - **shell completions**: `cargo gen completions`

###

## Pull requests
I have provided a [pull request template](.github/pull_request_template.md) to help guide in submitting a PR

