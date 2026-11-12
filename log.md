# Log

What this fork's work changed outside the repository. Newest day last.

## 2026-11-12

- Fork started from upstream `main` at `78f5d7446c` (1.30.0-dev), branch `fork`. Published as
  `ilyannn/zett` (public fork, Actions disabled, default branch `fork`); the remote is `zett`,
  `origin` stays upstream.
- Installed for the build: Rust 1.98.1 through rustup (from `rust-toolchain.toml`) and Xcode's
  Metal toolchain (`xcodebuild -downloadComponent MetalToolchain`). Reported to the machines
  agent, agent-mail thread #148.
- `just install` put `Zett.app` (`dev.zett.Zett`) into `~/Applications`; `just cli` linked
  `~/.local/bin/zett` to its CLI.
- Recent projects: Zett's database (`db/0-dev` in Zed's data folder) was replaced with a snapshot
  of the stable app's (`db/0-stable`); the previous one is kept beside it as
  `0-dev.before-import-2026-11-12`.
- Shared settings: asked the zed agent to set `disable_ai` and drop the Bedrock agent setup
  (thread #149, done), and to tint the zed-settings project with the key colour `#059669`
  (thread `2026-11-12-tint-the-zed-settings-project-window-in`).
- Agent directory: this folder's entry was renamed from `zed-source` to `zett`.
- The database backup `0-dev.before-import-2026-11-12` was moved to the Trash once the imported
  recent projects looked right. The folder moved to `~/Code/zett`.
