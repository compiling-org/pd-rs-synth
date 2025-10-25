# PD-RS Synth

A VST3 synthesizer plugin that integrates Pure Data patches as synthesizer modules. Based on the PlugData runtime system for embedded Pd patches in VST3 format.

## ⚠️ **WORK IN PROGRESS - Active Development**

### **Current Status & Recent Updates**

#### ✅ **Completed Features**
- **VST3 Framework**: Basic plugin structure and VST3 compatibility
- **PlugData Integration**: Pure Data runtime integration for patch execution
- **OSC Communication**: Open Sound Control support for parameter communication

#### 🔄 **In Development**
- **Extended Pd Library**: Custom objects for bio-signal processing
- **Real-time Performance**: Slive performance with Pd patches
- **Visual Programming**: Integration with Pd's visual patching interface

#### 🚧 **Known Issues**
- **VST3 Implementation**: Plugin interface and parameter handling incomplete
- **PlugData Integration**: Full Pd patch compatibility needs verification
- **Performance Optimization**: Real-time Pd execution optimization required
- **Host Compatibility**: Testing with major DAWs (Ableton, Logic, etc.)

#### 📈 **Next Development Phase**
1. **Complete VST3 Interface**: Finish plugin parameter and MIDI handling
2. **Full PlugData Integration**: Implement complete Pd patch loading and execution
3. **Performance Optimization**: Ensure sub-10ms latency for Pd patch execution
4. **DAW Integration Testing**: Verify compatibility with major hosts

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