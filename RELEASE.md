## Release Process

### Instructions

#### Quick

```bash
# *Important* Do not forget to replace `0.1.3` below by your real version!
export version=0.5.7

find . -type f -name Cargo.toml -not -path "./target/*" | xargs -n1 sd '^version = "[^"]*"' "version = \"${version}\""
find . -type f -name Cargo.toml -not -path "./target/*" | xargs -n1 sd '^substreams(-[^ =]+)?\s*=\s*\{\s*version\s*=\s*"[^"]+"' "substreams\$1 = { version = \"${version}\""
```

#### Detailed

> **Warning** Do not forget to replace `${version}` by your real version like `0.1.3` in the commands below!  (`export version=0.1.3`)

You will need [sfreleaser](https://github.com/streamingfast/sfreleaser) (install from source using Golang with `go install github.com/streamingfast/sfreleaser@latest`) to perform the release process.

- Find & replace all occurrences of Regex `^version = "[^"]+"` in all `Cargo.toml` files to `version = "${version}"`:
  Using [sd](https://github.com/chmln/sd):

  ```bash
  find . -type f -name Cargo.toml -not -path "./target/*" | xargs -n1 sd '^version = "[^"]*"' "version = \"${version}\""
  ```

- Find & replace all occurrences of Regex `^substreams(-[^ =]+)?\s*=\s*\{\s*version\s*=\s*"[^"]+"` in all `Cargo.toml` files to `substreams$1 = { version = "${version}"`

  Using [sd](https://github.com/chmln/sd):

  ```bash
  find . -type f -name Cargo.toml -not -path "./target/*" | xargs -n1 sd '^substreams(-[^ =]+)?\s*=\s*\{\s*version\s*=\s*"[^"]+"' "substreams\$1 = { version = \"${version}\""
  ```

- Update the [CHANGELOG.md](CHANGELOG.md) to update the `## Unreleased` header to become `## ${version}`:

  ```bash
  sd '## Unreleased' "## ${version}" CHANGELOG.md
  ```

- Perform a `cargo check` so that `Cargo.lock` is properly updated.

- Commit everything with message `Preparing release of ${version}`.

  ```bash
  git add -A . && git commit -m "Preparing release of ${version}"
  ```

- Perform release

  ```bash
  sfreleaser release
  ```