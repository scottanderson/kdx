# kdx-test-tables

Gives the workspace's tests the challenge tables. The tables are Haxial's
copyrighted text, so they are not in the repository. This crate is not
published; it exists only for the tests of the other crates in
[kdx](https://github.com/scottanderson/kdx/blob/main/README.md).

It reads a table from the `KDX_CHALLENGE_TABLE` or `HGC_CHALLENGE_TABLE`
environment variable (base64), from `vendor/haxial/`, or from the cache that
`kdx table fetch` fills, and fails without one. See [challenge
tables](https://github.com/scottanderson/kdx/blob/main/docs/setup/challenge-tables.md).

## License

Licensed under either of
[Apache License, Version 2.0](https://github.com/scottanderson/kdx/blob/main/LICENSE-APACHE)
or [MIT license](https://github.com/scottanderson/kdx/blob/main/LICENSE-MIT),
at your option.
