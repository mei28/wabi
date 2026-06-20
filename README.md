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
cargo run -- json
```

State is stored at:

```text
${XDG_STATE_HOME:-$HOME/.local/state}/wabi/state.json
```

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
