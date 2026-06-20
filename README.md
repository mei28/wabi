# wabi

`wabi` is a Rust CLI scaffold for rendering cached Claude and Codex rate
limit usage in terminal status surfaces.

The current implementation is a scaffold path. `wabi update` writes explicit
provider errors instead of calling real APIs, reading credentials, or spawning
`codex app-server`.

## Usage

```sh
just check
cargo run -- update
cargo run -- status
cargo run -- status --tmux
cargo run -- status --max-age 300
cargo run -- status --no-refresh
cargo run -- tick
cargo run -- json
```

State is stored at:

```text
${XDG_STATE_HOME:-$HOME/.local/state}/wabi/state.json
```

## Lazy refresh

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

Use these flags and commands to control refresh behavior:

- `wabi status --max-age <secs>` changes the TTL from the default 120 seconds.
- `wabi status --no-refresh` renders the cache without starting background work.
- `wabi tick --max-age <secs>` only triggers the lazy refresh check and prints
  nothing.

This replaces a resident launchd job; statusline and tmux callers can run
`wabi status` directly.

## Manual verification (TODO before finalizing parsers)

Run these only when you are ready to verify real response shapes:

```sh
curl -s https://api.anthropic.com/api/oauth/usage \
  -H "anthropic-beta: oauth-2025-04-20" \
  -H "authorization: Bearer <token>" | jq
```

```sh
codex app-server
# Then send JSON-RPC initialize and account/rateLimits/read messages over stdio.
```

Fields currently assumed by fixture parsers:

- Claude: `rate_limits.five_hour.used_percentage`,
  `rate_limits.five_hour.resets_at`,
  `rate_limits.seven_day.used_percentage`,
  `rate_limits.seven_day.resets_at`
- Codex: `result.primary.usedPercent`, `result.primary.resetsAt`,
  `result.secondary.usedPercent`, `result.secondary.resetsAt`
