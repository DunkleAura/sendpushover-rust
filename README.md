# sendpushover

A small Rust command-line client for sending [Pushover](https://pushover.net/) notifications.

## Build and check

Install a current stable Rust toolchain, then run:

```sh
cargo build --release
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The release executable is `target/release/sendpushover`.

### Multi-architecture release builds

The project uses [cross](https://github.com/cross-rs/cross) to build all supported targets through Docker or Podman. Install it once, then create the distributions:

```sh
make setup-cross
make dist
```

Docker or Podman must be running for targets that differ from the host. To build only selected targets, pass their names directly:

```sh
./scripts/build-all.sh linux-arm64
./scripts/build-all.sh linux-x86_64 windows-x86_64
```

`make dist` writes these executables:

```text
dist/sendpushover-linux-arm64
dist/sendpushover-linux-x86_64
dist/sendpushover-windows-x86_64.exe
```

The Linux ARM64 build works on general 64-bit ARM Linux systems, including Raspberry Pi 3 and newer running 64-bit Raspberry Pi OS. The build script is `scripts/build-all.sh`; set `CROSS` if the `cross` executable has a nonstandard name or path.

### Tagged releases

Pushing any Git tag starts `.github/workflows/release.yml`. The workflow creates a GitHub release and attaches all three platform binaries plus a versioned source `.tar.gz` archive.

```sh
git tag v0.3.0
git push origin v0.3.0
```

## Configuration

Set credentials with environment variables:

```sh
export PUSHOVER_TOKEN="your application token"
export PUSHOVER_USER="your user or group key"
```

Alternatively, use an INI file:

```ini
[api]
token = your application token
user = your user or group key
```

Configuration files are read in this order, with later values taking precedence:

1. `/etc/sendpushoverrc`
2. `./sendpushover.cfg`
3. `~/.sendpushoverrc`
4. Environment variables

## Usage

```sh
cargo run -- --message "Hello from Rust"

# Or use the release executable:
target/release/sendpushover --title "Build" --message "Build completed"
```

Run `sendpushover --help` for all options. `--verbose` prints diagnostics while redacting the API token.
