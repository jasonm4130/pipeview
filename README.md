# pipeview

Real-time pipeline debugger for your terminal. `pv` shows bytes — **pipeview shows your data**.

```
cat events.jsonl | pipeview | jq '.users[]' | grep "active" > out.txt
```

Drop `pipeview` into any shell pipeline to see what's flowing through: throughput, record samples, format detection, and more — all without touching your data.

## Install

```bash
cargo install pipeview
```

## Features

- **Transparent proxy** — data passes through untouched, byte-for-byte
- **Real-time TUI** — throughput sparkline, bandwidth, record count
- **Format detection** — auto-detects JSON, CSV, or plain text
- **Syntax highlighting** — colored output for JSON keys/values and CSV columns
- **Compact + fullscreen** — press `f` to toggle between modes
- **Quiet mode** — `--quiet` for scripted use, prints summary on completion

## Usage

### Interactive (TUI)

```bash
# Watch data flow through a pipeline
cat server.log | pipeview | grep ERROR > errors.txt

# Start in fullscreen mode
cat events.jsonl | pipeview --fullscreen | jq '.' > out.json

# Force a specific format
cat data.txt | pipeview --csv | cut -d',' -f1 > ids.txt
```

### Scripted (quiet mode)

```bash
$ cat huge.jsonl | pipeview -q | jq '.' > out.json
pipeview: 1,204,831 lines | 482MB | 14.2s | 33.9MB/s
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `f` | Toggle fullscreen / compact |
| `q` | Detach TUI, print summary |

## Options

```
pipeview [OPTIONS]

Options:
  -f, --fullscreen       Start in fullscreen mode
  -n, --sample-rate <N>  Show 1 in N records (default: auto)
  -b, --buffer <SIZE>    Ring buffer size (default: 8MB)
      --no-detect        Skip format detection
      --json             Force JSON mode
      --csv              Force CSV mode
  -q, --quiet            No TUI, print summary on completion
  -h, --help             Print help
  -V, --version          Print version
```

## How it works

```
stdin → [Reader Thread] → [Ring Buffer] → [Writer Thread] → stdout
                              ↓
                        [Stats Collector]
                              ↓
                        [TUI Renderer] → stderr
```

Three threads: reader pumps stdin into a shared buffer, writer drains to stdout, and the TUI samples stats to render on stderr. Your data never touches the rendering path.

## License

MIT
