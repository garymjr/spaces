# spaces

Lightweight CLI to create and manage named git clone "spaces" backed by a local mirror. Fast setup, clean isolation, and easy parallel work.

## Install

```bash
cargo install --path .
```

## Usage

```bash
spaces new my-space
spaces new my-space --branch feature/foo
spaces new my-space --branch feature/foo --from main
spaces list
spaces go my-space
spaces run my-space -- git status
spaces copy my-space -- ".env*" "*.json"
spaces mirrors
spaces mirrors update
spaces rm my-space
```

## Config

Config uses git config keys under `spaces.*` and a repo-local `.spacesrc` file.

Key paths:
- `spaces.clones.dir` (default: sibling `../<repo>-clones`)
- `spaces.clones.prefix`
- `spaces.mirrors.dir` (default: `~/.cache/spaces/mirrors/<repo>`)
- `spaces.defaultBranch`
- `spaces.copy.include`, `spaces.copy.exclude`
- `spaces.copy.includeDirs`, `spaces.copy.excludeDirs`
- `spaces.hook.postCreate`, `spaces.hook.preRemove`, `spaces.hook.postRemove`

## Notes

- Mirrors are updated on `spaces new` unless `--no-fetch` is set.
- `spaces mirrors update` forces a mirror update.
- `.worktreeinclude` and `.spacesinclude` are supported for copy patterns.

## License

MIT
