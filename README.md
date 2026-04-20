# create-video-effect

A CLI scaffold tool for generating PrGPU/VEKL-based Adobe video effect projects.

## Installation

```bash
cargo install create-video-effect
```

## Usage

```bash
# Create a single-pass effect (default)
cargo create-video-effect effect my_effect

# Create a multi-pass effect
cargo create-video-effect effect my_effect --mode multi-pass

# Specify target apps
cargo create-video-effect effect my_effect --app premiere
cargo create-video-effect effect my_effect --app afterfx
cargo create-video-effect effect my_effect --app premiere,afterfx

# Custom PIPL prefix (2–6 uppercase characters)
cargo create-video-effect effect my_effect --prefix MYFX

# Custom output directory
cargo create-video-effect effect my_effect --dir ./projects/my_effect
```

### Interactive Mode

If you omit required arguments, the CLI will prompt you interactively:

```bash
cargo create-video-effect
```

### Arguments

| Argument | Short | Description |
|---|---|---|
| `[type]` | — | Project type: `effect` (default) or `transition` |
| `[name]` | — | Crate name (must be a valid Rust identifier) |
| `--app` | `-a` | Target app(s): `premiere`, `afterfx` (default: both) |
| `--mode` | `-m` | Pass mode: `single-pass` (default) or `multi-pass` |
| `--prefix` | `-p` | PIPL match-name prefix (2–6 uppercase ASCII chars) |
| `--dir` | `-d` | Output directory override (default: `./<name>`) |

### Transitions

Transition generation is not yet implemented. Running with `transition` will produce an error.

## Generated Project Structure

### Single-Pass Effect

```
my_effect/
├── Cargo.toml
├── build.rs
├── rustfmt.toml
├── benches/
│   └── effect_cpu.rs
├── shaders/
│   └── my_effect.vekl
└── src/
    ├── kernel.rs
    ├── params.rs
    ├── lib.rs
    └── gpu.rs
```

### Multi-Pass Effect

```
my_effect/
├── Cargo.toml
├── build.rs
├── rustfmt.toml
├── benches/
│   └── effect_cpu.rs
├── shaders/
│   ├── my_effect_edge.vekl
│   └── my_effect_tint.vekl
└── src/
    ├── kernel.rs
    ├── params.rs
    ├── lib.rs
    └── gpu.rs
```

## Workspace Detection

If the output directory is inside a Cargo workspace, the tool automatically adds the new crate to the workspace's `members` list.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
