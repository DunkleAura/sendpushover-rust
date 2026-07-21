# sendpushover-rust

A small Rust command-line client for sending [Pushover](https://pushover.net/) notifications. The executable is named `sendpushover`.

> **Warning:** This is an old personal utility and is not production-ready. In particular, verbose output includes API request and response details and may expose credentials, message content, or other sensitive data.

## Features

- Reads Pushover credentials from environment variables or INI files
- Supports titles, priorities, devices, sounds, and supplementary URLs
- Can send quietly from scripts and cron jobs
- Builds for Linux x86-64, Linux ARM64, and Windows x86-64

## Requirements

You need a [Pushover account](https://pushover.net/) and:

- an application API token
- a user or group key

To build from source, install a current stable Rust toolchain.

## Installation

Download a binary from the repository's [GitHub releases](https://github.com/DunkleAura/sendpushover-rust/releases), or build it locally:

```sh
cargo build --release
```

The resulting executable is `target/release/sendpushover` (or `sendpushover.exe` on Windows). Copy it to a directory in your `PATH` if desired.

## Configuration

Credentials can be supplied through environment variables:

```sh
export PUSHOVER_TOKEN="your application token"
export PUSHOVER_USER="your user or group key"
```

Alternatively, create an INI file containing an `[api]` section:

```ini
[api]
token = your application token
user = your user or group key
```

Configuration is loaded in the following order, with later values taking precedence:

1. `/etc/sendpushoverrc`
2. `./sendpushover.cfg`
3. `~/.sendpushoverrc`
4. `PUSHOVER_TOKEN` and `PUSHOVER_USER`
5. `--user` for the user or group key

Empty values are treated as unset. Keep configuration files private because they contain credentials; for example, run `chmod 600 ~/.sendpushoverrc` on Unix-like systems.

## Usage

Send a basic notification:

```sh
sendpushover --message "Backup completed"
```

Set a title and priority:

```sh
sendpushover --title "Backup" --message "Backup completed" --priority 1
```

Target a device and choose a sound:

```sh
sendpushover \
  --message "Deployment completed" \
  --device phone \
  --sound magic
```

Add a supplementary URL:

```sh
sendpushover \
  --title "Build" \
  --message "Build completed" \
  --url "https://example.com/build/123" \
  --url-title "View build"
```

When running from the source tree, place application arguments after `--`:

```sh
cargo run -- --message "Hello from Rust"
```

### Options

| Option | Description |
| --- | --- |
| `-m, --message <MESSAGE>` | Message to display; required |
| `-s, --title <TITLE>` | Notification title; defaults to `user@hostname` |
| `-u, --user <KEY>` | User or group key; overrides configuration |
| `-p, --priority <VALUE>` | Pushover priority; defaults to `0` |
| `--device <DEVICE>` | Send only to the named device |
| `--sound <SOUND>` | Override the default notification sound |
| `--url <URL>` | Add a supplementary URL |
| `--url-title <TITLE>` | Set the supplementary URL title; requires `--url` |
| `-q, --quiet` | Suppress output; use the exit status to determine success |
| `-v, --verbose` | Print request and response diagnostics; conflicts with `--quiet` |
| `-h, --help` | Show command help |
| `-V, --version` | Show the version |

See the Pushover documentation for valid [priority values](https://pushover.net/api#priority) and [sound names](https://pushover.net/api#sounds).

### Output and exit status

On success, the command prints `Notification sent` and exits with status `0`. Configuration, network, and API errors are written to standard error and return status `1`. In quiet mode, both success and error messages are suppressed.

Use `--verbose` only while troubleshooting. Although the API token is currently redacted, diagnostics can still contain the user or group key, notification content, URLs, and API response data.

## Development

Run all project checks with:

```sh
make check
```

This runs the tests, formatting check, and Clippy with warnings treated as errors. The equivalent commands are:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Multi-architecture builds

The project uses [cross](https://github.com/cross-rs/cross) to build targets that differ from the host. Docker or Podman must be installed and running.

```sh
make setup-cross
make dist
```

To build only selected targets:

```sh
./scripts/build-all.sh linux-arm64
./scripts/build-all.sh linux-x86_64 windows-x86_64
```

Builds are written to:

```text
dist/sendpushover-linux-arm64
dist/sendpushover-linux-x86_64
dist/sendpushover-windows-x86_64.exe
```

Set the `CROSS` environment variable if the `cross` executable has a nonstandard name or path. The ARM64 build supports general 64-bit ARM Linux systems, including Raspberry Pi 3 and newer running a 64-bit operating system.

## Releases

Pushing a Git tag starts `.github/workflows/release.yml`, which creates a GitHub release containing the three platform binaries:

```sh
git tag v0.4.0
git push origin v0.4.0
```

## License

This project is licensed under the [MIT License](LICENSE).
