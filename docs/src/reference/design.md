# Design & Guarantees

The promises the extension makes, and the engineering behind them.

## In-memory, in-process, no I/O

Every operation takes byte buffers and returns byte buffers. The underlying
`c2pa` crate is compiled with `default-features = false`:

- **no `file_io`** — the extension cannot read or write files;
- **no `fetch_remote_manifests`** — a manifest that points at a remote store
  is not fetched; there are no trust-list downloads; and signing passes no
  timestamp-authority URL.
- **`rust_native_crypto`** — cryptography is pure Rust (no OpenSSL linkage),
  so the built library is portable across systems with differing SSL stacks.

The result: validating or signing an image is a bounded, stateless function
call. Nothing warms up, nothing persists, nothing talks to the outside world.
That makes the extension predictable inside a web request — the original
design constraint, coming from the WordPress upload pipeline.

## Byte-safety across the PHP ↔ Rust boundary

Image bytes are not valid UTF-8, and treating them as text corrupts them. On
the Rust side every byte parameter is `ext_php_rs::binary::Binary<u8>` — a
byte-preserving view of the PHP string — never a Rust `String`. On the PHP
side you pass ordinary strings; PHP strings are already binary-safe. The
practical rule: hand bytes straight from `file_get_contents()` (or your
storage layer) to the extension and back, with no encoding layer in between.

## Threading and SAPI posture

The extension is built and tested for PHP 8.3–8.5 NTS on Linux (x86_64,
arm64) and macOS (arm64); ZTS is declared supported but not yet exercised in
CI (no ZTS runner in the release matrix). It holds no global mutable state: each
`Reader`/`Builder`/`Signer` is an independent value, so concurrent requests
in any SAPI are fine.

## Error philosophy

One exception class (`C2paException`), typed error variants internally, and
a hard line between *verdicts* and *errors*: an unsigned image or a failed
validation is a verdict (a normal return), while corrupt input or a
misconfigured signer is an error (an exception). See [Errors](./errors.md).

## Versioning and releases

The crate follows semver via git tags (`v*`); each tag's CI run builds the
full platform × PHP-minor matrix and attaches:

- PIE-convention extension tarballs per leg, and
- a transitive third-party license manifest (`cargo about`) covering
  everything statically linked into the binaries.

Dependency versions are pinned exactly (`=x.y.z`) — upgrades are deliberate,
tested bumps, never silent drift.

## Licensing

GPL-2.0-or-later, © Automattic, Inc. — consistent with WordPress core. The
"or later" clause is load-bearing: the `c2pa` and `ext-php-rs` dependencies
are Apache-2.0/MIT, and Apache-2.0 is GPLv3-compatible, so distributing the
combined work exercises the GPLv3 option.

## Scaffolding

The repository is a child of
[DisplaceTech/ext-template](https://github.com/DisplaceTech/ext-template):
the build system, CI, docs, and release workflows are template-managed and
re-rendered with its `bin/sync` (driven by `.ext-template.conf` at the repo
root); the Rust sources, stubs, and this book are the extension's own.
