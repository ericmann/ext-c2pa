# ext-c2pa

PHP 8.3+ native C2PA Content Credentials: read/validate manifests and sign
Media Library images (incl. derivatives). PHP namespace `Automattic\VIP\C2PA`.

Part of the **wp-c2pa** VIP product (see the repository root `DESIGN.md`,
`PLAN.md`, and `CLAUDE.md`). The `plugin/` half consumes this extension; the
`demo/` harness proves both on a local vip-go site.

Repository: [github.com/ericmann/wp-c2pa](https://github.com/ericmann/wp-c2pa).
Scaffolded from [ext-template](https://github.com/DisplaceTech/ext-template);
the files listed in its `template/managed/` tree are kept in sync with
`bin/sync` — edit those upstream, not here. Everything else is this
extension's own and has been VIP-ified (namespace, license, copyright).
[ext-infer](https://github.com/DisplaceTech/ext-infer) is the worked
reference for house conventions (quality bar, layout, stubs).

## License

GPL-2.0-or-later © Automattic, Inc. — consistent with WordPress core. The
"or later" clause is load-bearing: it lets us link the Apache-2.0/MIT
dependencies (`c2pa`, `ext-php-rs`) by exercising the GPLv3 option. If
release binaries statically link third-party code, add
`THIRD-PARTY-NOTICES.md`; the release workflow attaches a transitive
`cargo about` manifest. Composer/PIE publishing is deferred (see `CLAUDE.md`).
