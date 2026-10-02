[speq-skill](../README.md) / [Docs](./index.md) / Fork sync

---

# Fork sync

This fork releases `vX.Y.Z-ar.N`: upstream release `vX.Y.Z` plus the fork's own commits. This page describes how to release a new fork version and how to move the fork onto a new upstream release.

## How the fork is organized

| Item | Meaning |
|------|---------|
| `upstream` remote | `marconae/speq-skill`, the original project |
| `origin` remote | `antonireus/speq-skill`, this fork |
| `main` branch | A copy of `upstream/main`. Nobody commits to it. |
| `ar` branch | The fork's release line and the default branch on GitHub. It holds an upstream release tag plus the fork's commits. |
| `vX.Y.Z-ar.N` tag | A fork release. Pushing it makes CI publish a GitHub release, and `install.sh` installs the latest one. |

The fork's commits on `ar` form a patch stack: a short, ordered list of commits that git replays onto each new upstream tag. Each commit covers one topic, so a conflict during the replay points at one change. The stack has this order, from the upstream tag up:

1. Changes upstream could accept. They come first, so they can be offered upstream as pull requests.
2. Fork preferences, such as model and effort pins.
3. `chore(fork): point install and repository links to antonireus/speq-skill`.
4. `chore(release): vX.Y.Z-ar.N`, always the last commit. It changes only the version in `Cargo.toml` and `Cargo.lock`.

The version uses a semver pre-release suffix. The suffix is `-ar.N` with a dot, because semver compares a dotted number as a number: `ar.10` sorts after `ar.2`, and `ar10` would sort before `ar2`. Semver also sorts `0.22.0-ar.1` before `0.22.0`. That is correct for this fork, because fork users install only fork releases.

## One-time setup in a new clone

```bash
git clone https://github.com/antonireus/speq-skill && cd speq-skill
git remote add upstream git@github.com:marconae/speq-skill.git
git fetch upstream --tags
git config rerere.enabled true
```

`rerere` makes git record how you resolve each conflict and reuse that resolution when the same conflict appears in a later rebase.

GitHub disables workflows on a new fork. Enable them once in the Actions tab. Without that, a tag push publishes no release.

## Work between releases

- Commit new work on top of `ar`, after the release commit. One topic per commit.
- To change an existing fork commit, commit the change with `git commit --fixup=<commit>`. Git names it `fixup! <subject>`, and the next release rebase merges it into that commit.
- Push with a plain `git push origin ar`. Only a release rewrites `ar`.
- To offer a change upstream, branch from `main`, cherry-pick the commit, and open a pull request against `marconae/speq-skill`.

## Release a new fork version on the same upstream release

Example: `v0.22.0-ar.1` to `v0.22.0-ar.2`.

```bash
git switch ar && git pull --ff-only
git rebase -i --autosquash v0.22.0
```

In the rebase editor, delete the line of the old `chore(release)` commit. `--autosquash` already places each `fixup!` commit under its target. Move new commits into the stack order above if needed. Then continue at [Bump, check, and publish](#bump-check-and-publish).

## Move the fork onto a new upstream release

Example: upstream publishes `v0.23.0`, and the fork is on `v0.22.0`.

```bash
git fetch upstream --tags
git switch main && git merge --ff-only upstream/main && git push origin main
git switch ar && git pull --ff-only
git rebase -i --autosquash --onto v0.23.0 v0.22.0 ar
```

In the rebase editor, delete the line of the old `chore(release)` commit.

- On a conflict, fix the files, run `git add <files>`, and run `git rebase --continue`. A conflict that `rerere` already knows is resolved automatically. Check the result before you continue.
- A fork commit that upstream adopted has nothing left to apply. Git drops it, or stops and reports an empty commit. When upstream adopted the change in a different form, the rebase stops with a conflict instead. In both stop cases, run `git rebase --skip` to drop the commit.

After the rebase, look for new upstream links that point at the original project or at `main`:

```bash
git grep -n "marconae/speq-skill"
git grep -n "antonireus/speq-skill/main"
```

The only expected match is the `authors` field in `Cargo.toml`. Fix any other match with a `fixup!` commit for the `chore(fork)` links commit, then run `git rebase -i --autosquash v0.23.0`.

## Bump, check, and publish

1. Set the version in `Cargo.toml`, for example `version = "0.23.0-ar.1"`. `Cargo.toml` is the only place that stores the version.
2. Run `cargo build` so that `Cargo.lock` picks up the new version.
3. Commit both files as the last commit: `git commit -am "chore(release): v0.23.0-ar.1"`.
4. Check the final state:

   ```bash
   ./scripts/plugin/build.sh
   cargo fmt --check && cargo clippy
   cargo test
   ```

5. Check every commit on its own, so that each one can be picked or bisected:

   ```bash
   git rebase -x './scripts/plugin/build.sh >/dev/null && cargo test -q >/dev/null' v0.23.0
   ```

   Do not pipe `cargo test` into another command inside `-x`. The pipe hides the test exit code, so a failing test passes the check.

6. Publish:

   ```bash
   git push --force-with-lease origin ar
   git tag -a v0.23.0-ar.1 -m "v0.23.0-ar.1"
   git push origin v0.23.0-ar.1
   gh run watch --repo antonireus/speq-skill
   ```

   CI runs the tests, builds four platform archives, and publishes the GitHub release.

## Check the release

```bash
gh release view v0.23.0-ar.1 --repo antonireus/speq-skill
curl -fsSL https://raw.githubusercontent.com/antonireus/speq-skill/ar/install.sh | bash
speq --version
```

The release lists four `speq-marketplace-v0.23.0-ar.1-<platform>.tar.gz` archives. `speq --version` prints `speq 0.23.0-ar.1`. Restart Claude Code to load the new plugin.
