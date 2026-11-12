# Zett, a personal build of Zed: run, bundle, install and sync with upstream. `just` lists recipes.

# Display name of the installed app: ~/Applications/<app_name>.app; matches the bundle-dev name in crates/zed/Cargo.toml.
app_name := "Zett"
# Name of the cli link in ~/.local/bin; must not be `zed`, which is the stable Zed's command.
cli_name := "zett"

# Cargo build scripts run cmake from directories outside this repo, where the mise shim has no
# pinned version, so put the real tool directories on PATH instead of relying on shims.
export PATH := shell("mise bin-paths | paste -sd: -") + ":" + env("PATH")

[private]
default:
    @just --list

# Run the dev build from source, in the release-fast profile (as .zed/tasks.json does)
run:
    cargo run --profile release-fast

# Quick type check of one package (default zed)
check package="zed":
    cargo check -p {{ package }}

# Build a release .app for this Mac, ad-hoc signed, no DMG; prints its path (long; touches crates/zed/Cargo.toml while running)
bundle:
    #!/usr/bin/env bb
    (ns bundle
      (:require [babashka.fs :as fs]
                [babashka.process :as p]))
    (p/shell "script/bundle-mac" "-l")
    (let [apps (fs/glob "target" "*/release/bundle/osx/*.app")
          newest (last (sort-by #(fs/file-time->millis (fs/last-modified-time %)) apps))]
      (if newest
        (println "Bundle:" (str (fs/absolutize newest)))
        (do (println "no .app found under target/*/release/bundle/osx") (System/exit 1))))

# Bundle, quit the running app, replace ~/Applications/<app_name>.app with the new bundle and launch it; never touches /Applications
install: bundle
    #!/usr/bin/env bb
    (ns install
      (:require [babashka.fs :as fs]
                [babashka.process :as p]
                [clojure.string :as str]))
    (def app-name "{{ app_name }}")
    (def expected-id
      (second (re-find #"(?s)\[package\.metadata\.bundle-dev\].*?identifier\s*=\s*\"([^\"]+)\""
                       (slurp "crates/zed/Cargo.toml"))))
    (defn bundle-id [app]
      (let [{:keys [exit out]} (p/shell {:out :string :err :string :continue true}
                                        "/usr/libexec/PlistBuddy" "-c" "Print :CFBundleIdentifier"
                                        (str (fs/path app "Contents" "Info.plist")))]
        (when (zero? exit) (str/trim out))))
    (def installed-binary
      (str (fs/path (fs/home) "Applications" (str app-name ".app") "Contents" "MacOS" "zed")))
    (defn running? []
      (zero? (:exit (p/shell {:out :string :err :string :continue true} "pgrep" "-f" installed-binary))))
    (defn die [& msg] (println (apply str msg)) (System/exit 1))
    (when-not expected-id (die "cannot read the bundle-dev identifier from crates/zed/Cargo.toml"))
    (let [apps (fs/glob "target" "*/release/bundle/osx/*.app")
          source (last (sort-by #(fs/file-time->millis (fs/last-modified-time %)) apps))
          target (fs/path (fs/home) "Applications" (str app-name ".app"))]
      (when-not source (die "no bundle found; run `just bundle`"))
      (when-not (= expected-id (bundle-id source))
        (die "refusing: " source " does not have identifier " expected-id))
      (when (or (fs/sym-link? target) (fs/exists? target))
        (when-not (and (fs/directory? target) (not (fs/sym-link? target))
                       (= expected-id (bundle-id target)))
          (die "refusing to replace " target ": not an app bundle with identifier " expected-id)))
      (when (running?)
        (println "Quitting running" app-name)
        (p/shell "osascript" "-e" (str "tell application id \"" expected-id "\" to quit"))
        (loop [n 0]
          (when (and (running?) (< n 50))
            (Thread/sleep 200)
            (recur (inc n))))
        (when (running?) (die "the app did not quit; quit it and run again")))
      (fs/create-dirs (fs/parent target))
      (when (fs/exists? target) (fs/delete-tree target))
      (p/shell "ditto" (str source) (str target))
      (println "Installed" (str target))
      (p/shell "open" (str target)))

# Link the installed app's cli into ~/.local/bin/<cli_name>; refuses to replace anything but a symlink into our app
cli:
    #!/usr/bin/env bb
    (ns cli
      (:require [babashka.fs :as fs]))
    (let [app (fs/path (fs/home) "Applications" "{{ app_name }}.app")
          source (fs/path app "Contents" "MacOS" "cli")
          target (fs/path (fs/home) ".local" "bin" "{{ cli_name }}")]
      (when-not (fs/exists? source)
        (println "missing" (str source) "- run `just install` first")
        (System/exit 1))
      (when (or (fs/sym-link? target) (fs/exists? target))
        (when-not (and (fs/sym-link? target)
                       (.startsWith (str (fs/read-link target)) (str app "/")))
          (println "refusing to replace" (str target) "- not a symlink into" (str app))
          (System/exit 1))
        (fs/delete target))
      (fs/create-dirs (fs/parent target))
      (fs/create-sym-link target source)
      (println "linked" (str target) "->" (str source)))

# Fetch upstream and rebase fork onto origin/main; stops if tracked files have uncommitted changes; pushes nothing
sync:
    #!/usr/bin/env bb
    (ns sync
      (:require [babashka.process :as p]
                [clojure.string :as str]))
    (defn git [& args] (apply p/shell {:out :string :err :string :continue true} "git" args))
    (let [dirty (str/trim (:out (git "status" "--porcelain" "--untracked-files=no")))
          branch (str/trim (:out (git "branch" "--show-current")))]
      (when-not (= branch "fork")
        (println "refusing: on branch" (pr-str branch) "- check out fork first")
        (System/exit 1))
      (when-not (str/blank? dirty)
        (println "working tree has uncommitted changes; commit or stash them first:")
        (println dirty)
        (System/exit 1))
      (p/shell "git" "fetch" "origin")
      (p/shell "git" "rebase" "origin/main"))

# List the fork's own commits and changed files relative to origin/main
patches:
    git log --oneline origin/main..HEAD
    git diff --stat origin/main...HEAD

# Clippy through the repo's script, which the repo requires instead of cargo clippy (long: whole workspace)
lint:
    ./script/clippy

# Run tests with nextest, as CI does; `just test workspace` for one package (long without a package)
test package="":
    cargo nextest run {{ if package == "" { "--workspace" } else { "-p " + package } }} --no-fail-fast --no-tests=warn

# Format all crates
fmt:
    cargo fmt --all
