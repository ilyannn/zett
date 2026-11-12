> [!IMPORTANT]
> This is a fork, not Zed. Please do not open pull requests from it against
> [zed-industries/zed](https://github.com/zed-industries/zed): its changes are personal and are
> not meant for upstream. To contribute to Zed, start from upstream's `main`.

# Zett

Zett is a personal fork of [Zed](https://github.com/zed-industries/zed), built for one Mac. It is
not affiliated with or supported by Zed Industries; for the editor itself, use upstream. The
branch `fork` is rebased onto upstream `main`, so its history is rewritten regularly.

What differs from upstream:

- **Window colors per project.** A project's `.zed/settings.json` can set
  `window_theme_overrides`, in the same form as `theme_overrides`, to recolor the windows that
  show that project. Syntax styles are shared by all windows and are not affected.
- **No telemetry.** Events, crash handling and minidump uploads are compiled out, and update
  requests carry no machine or metrics ids.
- **No AI features.** `disable_ai` is always on, whatever the settings say.
- **Its own name and icon.** The dev channel builds as `Zett.app` (`dev.zett.Zett`) and runs next
  to an installed Zed, sharing its configuration in `~/.config/zed`.
- **A mise and just harness.** `mise install`, then `just` lists the recipes: `bundle`,
  `install` (into `~/Applications`), `cli`, `sync` (rebase onto upstream), `patches`.

No binaries are published. Upstream's README follows.

# Zed

[![Zed](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/zed-industries/zed/main/assets/badge/v0.json)](https://zed.dev)
[![CI](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml/badge.svg)](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml)

Welcome to Zed, a high-performance, multiplayer code editor from the creators of [Atom](https://github.com/atom/atom) and [Tree-sitter](https://github.com/tree-sitter/tree-sitter).

---

### Installation

On macOS, Linux, and Windows you can [download Zed directly](https://zed.dev/download) or install Zed via your local package manager ([macOS](https://zed.dev/docs/installation#macos)/[Linux](https://zed.dev/docs/linux#installing-via-a-package-manager)/[Windows](https://zed.dev/docs/windows#package-managers)).

Other platforms are not yet available:

- Web ([tracking discussion](https://github.com/zed-industries/zed/discussions/26195))

### Developing Zed

- [Building Zed for macOS](./docs/src/development/macos.md)
- [Building Zed for Linux](./docs/src/development/linux.md)
- [Building Zed for Windows](./docs/src/development/windows.md)

### Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for ways you can contribute to Zed.

Also... we're hiring! Check out our [jobs](https://zed.dev/jobs) page for open roles.

### Licensing

Zed source code is licensed primarily under GPL-3.0-or-later, with Apache-2.0 components where marked.

License information for third party dependencies must be correctly provided for CI to pass.

We use [`cargo-about`](https://github.com/EmbarkStudios/cargo-about) to automatically comply with open source licenses. If CI is failing, check the following:

- Is it showing a `no license specified` error for a crate you've created? If so, add `publish = false` under `[package]` in your crate's Cargo.toml.
- Is the error `failed to satisfy license requirements` for a dependency? If so, first determine what license the project has and whether this system is sufficient to comply with this license's requirements. If you're unsure, ask a lawyer. Once you've verified that this system is acceptable add the license's SPDX identifier to the `accepted` array in `script/licenses/zed-licenses.toml`.
- Is `cargo-about` unable to find the license for a dependency? If so, add a clarification field at the end of `script/licenses/zed-licenses.toml`, as specified in the [cargo-about book](https://embarkstudios.github.io/cargo-about/cli/generate/config.html#crate-configuration).

## Sponsorship

Zed is developed by **Zed Industries, Inc.**, a for-profit company.

If you’d like to financially support the project, you can do so via GitHub Sponsors.
Sponsorships go directly to Zed Industries and are used as general company revenue.
There are no perks or entitlements associated with sponsorship.
