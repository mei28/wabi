# wabi

`wabi` renders cached Claude and Codex rate-limit usage for terminal status
surfaces such as statuslines and tmux.

It targets macOS and Unix-like systems. Claude usage depends on the macOS
Keychain, and background refresh uses Unix process behavior.

## Install

After crates.io publication:

```sh
cargo install wabi
```

From source:

```sh
cargo build --release
```

The built binary is available at `target/release/wabi`.

## Data Sources

`wabi update` collects each provider independently and writes one cached state
file. If a provider fails, the error is recorded for that provider and the
state file is still updated.

- Claude usage is read from Anthropic's OAuth usage endpoint. `wabi` obtains
  the Claude Code OAuth token from the macOS Keychain service
  `Claude Code-credentials`.
- Codex usage is read by spawning `codex app-server` and requesting account
  rate limits through its JSON-RPC interface.

## Commands

```sh
wabi update
wabi status
wabi status --tmux
wabi status --max-age 300
wabi status --no-refresh
wabi tick
wabi tick --max-age 300
wabi json
```

- `wabi update` refreshes provider data and writes the cache.
- `wabi status` prints the cached state using ANSI color.
- `wabi status --tmux` prints tmux-compatible color formatting.
- `wabi status --max-age <secs>` changes the cache TTL from the default 120
  seconds.
- `wabi status --no-refresh` prints the cache without starting background
  refresh.
- `wabi tick --max-age <secs>` performs only the lazy refresh check and prints
  nothing.
- `wabi json` prints the cached state as formatted JSON.

## State

State is stored at:

```text
${XDG_STATE_HOME:-$HOME/.local/state}/wabi/state.json
```

`wabi status` always prints the cached state immediately. By default, it also
checks the cache age before rendering. If `collected_at` is older than 120
seconds, or if the cache does not exist, it starts a detached `wabi update` in
the background and returns without waiting for network or provider work.

`wabi update` uses a non-blocking lock at:

```text
${XDG_STATE_HOME:-$HOME/.local/state}/wabi/update.lock
```

If another update already holds the lock, the command exits successfully
without doing duplicate provider work. The lock is released by the operating
system when the process exits.

This replaces a resident launchd job. Statusline and tmux callers can run
`wabi status` directly.
