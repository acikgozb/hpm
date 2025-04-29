# `hpm`


[![CI](https://github.com/acikgozb/hpm/actions/workflows/ci.yml/badge.svg)](https://github.com/acikgozb/hpm/actions/workflows/ci.yml) ![version](https://img.shields.io/badge/version-0.1.0-red) ![release](https://img.shields.io/badge/release-stable-89e051)

A simple, wrapper CLI for host power management.
Uses `systemctl` and `loginctl`.
It is intentionally designed to be run on Linux hosts.

This is a more robust version of the [`devtools/power`](https://github.com/acikgozb/devtools/blob/main/de/power) PoC script.

## Table of Contents 

<!--toc:start-->
  - [Installation](#installation)
    - [Build From Source](#build-from-source)
    - [Prebuilt Binaries](#prebuilt-binaries)
  - [Usage](#usage)
  - [LICENSE](#license)
<!--toc:end-->

## <a id='installation'></a> Installation

Whilst `hpm` can be built for every platform, it is recommended to install on Linux hosts due having hard dependencies on `systemctl` and `loginctl`.

As usual, there are 2 ways to install `hpm`:

### <a id='build-from-source'></a> Build From Source

If you have `cargo` installed on your host, you can use it to build `hpm` from source.

```bash
# Clone the repository.
git clone git@github.com:acikgozb/hpm.git ./hpm

# Install via `cargo`.
cd ./hpm
cargo build --release --locked 

# Put the binary under $PATH.
# In here, it is assumed that ~/.local/bin is on $PATH.
cp ./target/release/hpm ~/.local/bin/hpm

# Validate the $PATH lookup before using hpm.
which hpm
```

### <a id='prebuilt-binaries'></a> Prebuilt Binaries

You can also install `hpm` by downloading prebuilt binaries from the [releases page](https://github.com/acikgozb/hpm/releases).

Extract `hpm` from its archive, and then put it under `$PATH` like above.

## <a id='usage'></a> Usage

```bash
# Check help before starting out.
hpm -h

# Shutdown.
hpm kill

# Restart.
hpm restart

# Logout from $USER.
hpm logout

# Interactive mode.
# Gives you a prompt to select the command to run.
hpm <-i, --interactive>
```

## <a id='license'></a> LICENSE

This work is dual-licensed under Apache 2.0 and GPL 2.0 (or any later version).
You can choose between one of them if you use this work.

`SPDX-License-Identifier: Apache-2.0 OR GPL-2.0-or-later`
