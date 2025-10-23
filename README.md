# PD-RS Synth

A VST3 synthesizer plugin that integrates Pure Data patches as synthesizer modules. Based on the PlugData runtime system for embedded Pd patches in VST3 format.

## Features

- VST3 plugin framework
- Pure Data integration via PlugData runtime
- OSC communication support
- Modular synthesizer architecture

## Building

```bash
cargo build --release
```

## Usage

This plugin allows you to load and run Pure Data patches within a VST3 host environment, providing a bridge between the visual programming paradigm of Pure Data and professional audio production workflows.

## License

MIT