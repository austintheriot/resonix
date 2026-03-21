# Compare Benchmarks

Runs a simple Rust script to compare previously-generated baseline benchmarks results against newer (probably branch-specific) benchmarks.

To generate new baseline benchmarks:

```sh
cargo criterion --message-format=json > benches/baseline.json
```

To generate a fresh, temporary benchmark to compare against:

```sh
cargo criterion --message-format=json > benches/new.json
```

To replace old baseline with the new one benchmark:

```sh
cp benches/new.json benches/baseline.json
```

To compare the existing benchmarks against the new one:

```sh
cargo run -p compare-benchmarks
```

To see an in-depth HTML-formatted report, see: `target/criterion/reports/index.htm`
