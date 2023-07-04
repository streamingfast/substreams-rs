## Release Process

### Instructions

Requires `Bash`, [sfreleaser](https://github.com/streamingfast/sfreleaser) and [sd](https://github.com/chmln/sd):

```bash
# *Important* Do not forget to replace `0.1.3` below by your real version!
export version=0.5.7

sd '^version = ".*?"$' "version = \"${version}\"" Cargo.toml
sd 'version = ".*?",' "version = \"${version}\"," Cargo.toml
sd '## Unreleased' "## ${version}" CHANGELOG.md

git add -A . && git commit -m "Preparing release of ${version}"

sfreleaser release
```
