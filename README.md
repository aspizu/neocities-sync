# neocities-sync

Sync a directory to neocities.

For every file inside a directory, upload them to neocities only if they have been
modified. Delete files which exist on neocities but not locally.

Creates a `.sync` file to store SHA1 hashes of files. If you get a out-of-sync error,
delete this file, and the hashes will be fetched from neocities.

## Installation

No need to install, just run

```bash
bunx github:aspizu/neocities-sync
```

## Usage

```bash
neocities-sync --help
```

## Development

```bash
git clone https://github.com/aspizu/neocities-sync
cd neocities-sync
bun run src/index.ts
```

## Contributing

Pull requests are welcome.
