# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic
Versioning.

## [0.1.1] - 2026-06-21

### Fixed

- Preserve full anyhow error chain in provider error state instead of only the outermost context message.

## [0.1.0] - 2026-06-20

### Added

- Initial release of the Claude and Codex rate-limit dashboard CLI.
- Cached state with lazy, debounced background refresh.
- tmux and ANSI statusline rendering.
- JSON output for the cached state.
