# Installation

ext-c2pa targets **PHP 8.3+** (NTS and ZTS) on Linux and macOS. Windows is
not supported.

## Pre-built binaries

Every tagged release attaches pre-built extension tarballs to the
[GitHub Release](https://github.com/ericmann/ext-c2pa/releases), named with
[PIE](https://github.com/php/pie)'s filename convention per platform and PHP
minor:

```text
php_c2pa-v0.1.0_php8.5-arm64-darwin-bsdlibc-nts.tgz
php_c2pa-v0.1.0_php8.4-x86_64-linux-glibc-nts.tgz
php_c2pa-v0.1.0_php8.3-arm64-linux-glibc-nts.tgz
```

Each tarball contains the compiled extension. Unpack it somewhere your PHP
can read and point your `php.ini` at it:

```ini
extension=/path/to/libc2pa.so   ; .dylib on macOS
```

Verify:

```console
$ php -m | grep c2pa
c2pa
```

> The package is not yet listed on Packagist/PIE, so `pie install
> ericmann/ext-c2pa` does not resolve yet — grab the tarball from the
> release directly.

## Building from source

You need:

- **Rust** (the pinned toolchain in `rust-toolchain.toml` is installed
  automatically by `rustup`)
- **PHP 8.3+** with `php-config` on your `PATH` (the `php-dev`/`php-devel`
  package on most Linux distributions)
- Linux: `build-essential libclang-dev` — macOS: Xcode command-line tools

Then:

```console
$ git clone https://github.com/ericmann/ext-c2pa
$ cd ext-c2pa
$ make build      # debug build → target/debug/libc2pa.{so,dylib}
$ make test       # PHPT suite against the just-built extension
$ make release    # optimized build → target/release/libc2pa.{so,dylib}
```

Load the built library the same way as above, or one-off on the CLI:

```console
$ php -d extension=target/debug/libc2pa.dylib -r 'var_dump(extension_loaded("c2pa"));'
bool(true)
```

`make help` lists the other targets (`clippy`, `fmt`, `stubs`,
`install`/`uninstall` via `cargo-php`, `clean`).

## IDE stubs

`stubs/c2pa.stubs.php` mirrors the full PHP surface for IDEs and static
analyzers (PHPStan, Psalm, IntelliSense). Point your tooling at it — the
extension itself ships no PHP files.
