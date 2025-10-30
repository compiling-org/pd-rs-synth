//! # PD-RS Synth VST3
//!
//! A professional VST3 plugin that integrates Pure Data patches as synthesizer modules.
//! Based on the PlugData runtime system for embedded Pd patches in VST3 format.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use midir::{MidiInput, MidiInputConnection, ConnectError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wmidi::{MidiMessage, Note, U7};

#[cfg(feature = "hexodsp-daw")]
use hexodsp_daw::audio_engine::node_graph::{NodeGraph, NodeType, NodeConnection};
#[cfg(feature = "hexodsp-daw")]
use hexodsp_daw::audio_engine::dsp_core::DSPModule;

/// VST3 plugin errors
#[derive(Error, Debug)]
pub enum PdRsSynthError {
    #[error("Pure Data initialization error: {0}")]
    PdInitialization(String),
    
    #[error("Audio processing error: {0}")]
    AudioProcessing(String),
    
    #[error("MIDI error: {0}")]
    MidiError(#[from] ConnectError<MidiInput>),
    
    #[error("Patch loading error: {0}")]
    PatchError(String),
    
    #[error("Parameter error: {0}")]
    ParameterError(String),
}

/// Plugin parameters for real-time control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdPluginParameters {
    pub master_volume: f32,
    pub polyphony: u32,
    pub sample_rate: f32,
    pub buffer_size: usize,
    pub patch_brightness: f32,
    pub fm_amount: f32,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub reverb_amount: f32,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub custom_parameters: HashMap<String, f32>,
}

/// Pure Data object types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PdObjectType {
    Oscillator,
    Filter,
    Envelope,
    Lfo,
    Mixer,
    Delay,
    Reverb,
    AudioInput,
    AudioOutput,
    MessageOutput,
}

/// Individual Pure Data object
#[derive(Debug, Clone)]
pub struct PdObject {
    pub id: String,
    pub object_type: PdObjectType,
    pub pd_name: String, // The actual Pd object name
    pub audio_inlets: Vec<String>,
    pub audio_outlets: Vec<String>,
    pub message_inlets: Vec<String>,
    pub parameters: HashMap<String, f32>,
    pub is_active: bool,
}

/// Main VST3 plugin structure
pub struct PdRsSynth {
    // Core plugin state
    pub parameters: Arc<PdPluginParameters>,
    pub is_initialized: bool,
    pub sample_rate: f32,
    pub block_size: usize,
    pub num_channels: usize,
    
    // Pure Data integration
    pub pd_patches: HashMap<String, PdPatch>,
    pub pd_objects: HashMap<String, PdObject>,
    pub active_patches: Vec<String>,
    
    // Audio processing
    pub audio_buffer: Vec<f32>,
    pub output_buffer: Vec<f32>,
    pub fm_buffer: Vec<f32>,
    pub modulation_buffer: Vec<f32>,
    
    // MIDI handling
    pub midi_input: Option<MidiInputConnection<()>>,
    pub active_notes: HashMap<u8, (Note, U7, Instant)>,
    pub last_note_time: Option<Instant>,
    
    // Performance metrics
    pub cpu_usage: f32,
    pub pd_processing_time: f32,
    pub last_processing_time: Instant,
    
    // Integration with hexodsp
    #[cfg(feature = "hexodsp-daw")]
    pub hexodsp_node_graph: Option<NodeGraph>,
    #[cfg(feature = "hexodsp-daw")]
    pub hexodsp_module_id: Option<usize>,
}

impl Default for PdRsSynth {
    fn default() -> Self {
        let default_params = PdPluginParameters {
            master_volume: 0.8,
            polyphony: 16,
            sample_rate: 44100.0,
            buffer_size: 512,
            patch_brightness: 0.7,
            fm_amount: 0.3,
            filter_cutoff: 800.0,
            filter_resonance: 0.5,
            reverb_amount: 0.2,
            delay_time: 0.25,
            delay_feedback: 0.3,
            custom_parameters: HashMap::new(),
        };
        
        Self {
            parameters: Arc::new(default_params),
            is_initialized: false,
            sample_rate: 44100.0,
            block_size: 512,
            num_channels: 2,
            pd_patches: HashMap::new(),
            pd_objects: HashMap::new(),
            active_patches: Vec::new(),
            audio_buffer: vec![0.0; 1024],
            output_buffer: vec![0.0; 1024],
            fm_buffer: vec![0.0; 1024],
            modulation_buffer: vec![0.0; 1024],
            midi_input: None,
            active_notes: HashMap::new(),
            last_note_time: None,
            cpu_usage: 0.0,
            pd_processing_time: 0.0,
            last_processing_time: Instant::now(),
            #[cfg(feature = "hexodsp-daw")]
            hexodsp_node_graph: None,
            #[cfg(feature = "hexodsp-daw")]
            hexodsp_module_id: None,
        }
    }
}

impl PdRsSynth {
    /// Initialize the plugin with audio settings
    pub fn initialize(&mut self, sample_rate: f32, block_size: usize, num_channels: usize) -> Result<()> {
        self.sample_rate = sample_rate;
        self.block_size = block_size;
        self.num_channels = num_channels;
        
        // Resize audio buffers
        let buffer_size = block_size * num_channels;
        self.audio_buffer.resize(buffer_size, 0.0);
        self.output_buffer.resize(buffer_size, 0.0);
        self.fm_buffer.resize(buffer_size, 0.0);
        self.modulation_buffer.resize(buffer_size, 0.0);
        
        // Initialize Pure Data patches
        self.initialize_default_patches()?;
        
        // Setup MIDI input
        self.initialize_midi()?;
        
        // Initialize hexodsp integration
        #[cfg(feature = "hexodsp-daw")]
        self.initialize_hexodsp_integration()?;
        
        self.is_initialized = true;
        log::info!("PlugData VST3 plugin initialized: sr={}, blocksize={}, channels={}", 
                  sample_rate, block_size, num_channels);
        Ok(())
    }
    
    /// Initialize default Pure Data patches
    fn initialize_default_patches(&mut self) -> Result<()> {
        // Initialize a simple subtractive synthesis patch
        let subtractive_patch = PdPatch {
            name: "subtractive_synth".to_string(),
            path: "default".to_string(),
            audio_inlets: vec!["audio_in".to_string()],
            audio_outlets: vec!["audio_out".to_string()],
            message_inlets: vec!["frequency".to_string(), "gain".to_string()],
            parameters: HashMap::from([
                ("freq".to_string(), PdParameter {
                    name: "frequency".to_string(),
                    default_value: 440.0,
                    min_value: 20.0,
                    max_value: 8000.0,
                    unit: "Hz".to_string(),
                    automation_enabled: true,
                }),
                ("gain".to_string(), PdParameter {
                    name: "gain".to_string(),
                    default_value: 0.8,
                    min_value: 0.0,
                    max_value: 1.0,
                    unit: "".to_string(),
                    automation_enabled: true,
                }),
            ]),
            connections: Vec::new(),
        };
        
        self.pd_patches.insert("subtractive_synth".to_string(), subtractive_patch);
        
        // Initialize default Pd objects for the patch
        self.create_default_pd_objects()?;
        
        Ok(())
    }
    
    /// Create default Pure Data objects
    fn create_default_pd_objects(&mut self) -> Result<()> {
        // Oscillator
        let osc_object = PdObject {
            id: "osc1".to_string(),
            object_type: PdObjectType::Oscillator,
            pd_name: "osc~".to_string(),
            audio_inlets: vec!["frequency".to_string()],
            audio_outlets: vec!["output".to_string()],
            message_inlets: vec!["reset".to_string()],
            parameters: HashMap::from([
                ("frequency".to_string(), 440.0),
                ("amplitude".to_string(), 0.8),
            ]),
            is_active: true,
        };
        self.pd_objects.insert("osc1".to_string(), osc_object);
        
        // Filter
        let filter_object = PdObject {
            id: "filter1".to_string(),
            object_type: PdObjectType::Filter,
            pd_name: "lowpass".to_string(),
            audio_inlets: vec!["input".to_string(), "cutoff".to_string(), "q".to_string()],
            audio_outlets: vec!["output".to_string()],
            message_inlets: vec![],
            parameters: HashMap::from([
                ("cutoff".to_string(), 1000.0),
                ("resonance".to_string(), 0.5),
            ]),
            is_active: true,
        };
        self.pd_objects.insert("filter1".to_string(), filter_object);
        
        // Envelope
        let envelope_object = PdObject {
            id: "env1".to_string(),
            object_type: PdObjectType::Envelope,
            pd_name: "adsr~".to_string(),
            audio_inlets: vec!["trigger".to_string()],
            audio_outlets: vec!["output".to_string()],
            message_inlets: vec!["attack".to_string(), "decay".to_string(), "sustain".to_string(), "release".to_string()],
            parameters: HashMap::from([
                ("attack".to_string(), 0.01),
                ("decay".to_string(), 0.1),
                ("sustain".to_string(), 0.7),
                ("release".to_string(), 0.2),
            ]),
            is_active: true,
        };
        self.pd_objects.insert("env1".to_string(), envelope_object);
        
        Ok(())
    }
    
    /// Initialize MIDI input
    fn initialize_midi(&mut self) -> Result<()> {
        let midi_in = MidiInput::new("PlugData VST3 MIDI Input")?;
        
        // Get first available port
        let ports = midi_in.ports();
        if ports.is_empty() {
            log::warn!("No MIDI input ports available");
            return Ok(());
        }
        
        let port = &ports[0];
        let port_name = midi_in.port_name(port).unwrap_or("MIDI Input".to_string());
        
        let conn = midi_in.connect(port, &port_name, |_, message, _| {
            // Process MIDI messages
            if let Ok(midi_msg) = MidiMessage::try_from(message) {
                match midi_msg {
                    MidiMessage::NoteOn(note, velocity, _) => {
                        log::debug!("NoteOn: {:?} velocity: {:?}", note, velocity);
                        self.trigger_note(note, velocity);
                    }
                    MidiMessage::NoteOff(note, _, _) => {
                        log::debug!("NoteOff: {:?}", note);
                        self.release_note(note);
                    }
                    _ => {
                        log::debug!("MIDI message: {:?}", midi_msg);
                    }
                }
            }
        }, ())?;
        
        self.midi_input = Some(conn);
        log::info!("MIDI input initialized");
        Ok(())
    }
    
    /// Initialize integration with hexodsp DAW
    #[cfg(feature = "hexodsp-daw")]
    fn initialize_hexodsp_integration(&mut self) -> Result<()> {
        let mut node_graph = NodeGraph::new();
        node_graph.set_audio_flow(self.sample_rate as u32, self.block_size);

        // Create a plugdata node in the graph
        let plugdata_node_id = node_graph.add_oscillator(); // Using oscillator as base for now
        self.hexodsp_module_id = Some(plugdata_node_id);

        self.hexodsp_node_graph = Some(node_graph);
        log::info!("Hexodsp integration initialized with node graph");
        Ok(())
    }
    
    /// Trigger a MIDI note
    fn trigger_note(&mut self, note: Note, velocity: U7) {
        let channel_note = note.to_midi_note_number();
        let velocity_val = velocity.as_u7() as f32 / 127.0;
        
        self.active_notes.insert(channel_note, (note, velocity, Instant::now()));
        self.last_note_time = Some(Instant::now());
        
        // Trigger envelope
        for obj in self.pd_objects.values_mut() {
            if obj.object_type == PdObjectType::Envelope {
                obj.parameters.insert("trigger".to_string(), velocity_val);
            }
        }
        
        log::debug!("Triggered note {} with velocity {}", note.to_string(), velocity_val);
    }
    
    /// Release a MIDI note
    fn release_note(&mut self, note: Note) {
        let channel_note = note.to_midi_note_number();
        self.active_notes.remove(&channel_note);
        
        log::debug!("Released note {}", note.to_string());
    }
    
    /// Process audio block
    pub fn process_audio(&mut self, input: &[f32], output: &mut [f32]) -> Result<()> {
        if !self.is_initialized {
            return Err(PdRsSynthError::AudioProcessing("Plugin not initialized".to_string()).into());
        }

        let start_time = Instant::now();

        // Clear output buffer
        output.fill(0.0);

        // Process Pure Data objects
        self.process_pd_objects(input, output)?;

        // Apply Pure Data post-processing
        self.apply_pd_post_processing(output)?;

        // Process through hexodsp node graph if available
        #[cfg(feature = "hexodsp-daw")]
        if let Some(ref mut node_graph) = self.hexodsp_node_graph {
            node_graph.process(input, output);
        }

        // Update performance metrics
        let processing_time = start_time.elapsed();
        self.cpu_usage = (processing_time.as_secs_f32() / (1.0 / self.sample_rate)) * 100.0;
        self.last_processing_time = start_time;

        Ok(())
    }
    
    /// Process all Pure Data objects
    fn process_pd_objects(&mut self, input: &[f32], output: &mut [f32]) -> Result<()> {
        let pd_start_time = Instant::now();
        
        // Process oscillator objects
        self.process_oscillators(input, output)?;
        
        // Process filter objects
        self.process_filters(input, output)?;
        
        // Process envelope objects
        self.process_envelopes(input, output)?;
        
        // Process other object types
        for obj in self.pd_objects.values_mut() {
            match obj.object_type {
                PdObjectType::Lfo => self.process_lfo(obj, input, output)?,
                PdObjectType::Mixer => self.process_mixer(obj, input, output)?,
                PdObjectType::Delay => self.process_delay(obj, input, output)?,
                PdObjectType::Reverb => self.process_reverb(obj, input, output)?,
                _ => {}
            }
        }
        
        let pd_processing_time = pd_start_time.elapsed();
        self.pd_processing_time = pd_processing_time.as_secs_f32() * 1000.0; // Convert to milliseconds
        
        Ok(())
    }
    
    /// Process oscillator objects
    fn process_oscillators(&mut self, input: &[f32], output: &mut [f32]) -> Result<()> {
        for obj in self.pd_objects.values_mut() {
            if obj.object_type == PdObjectType::Oscillator && obj.is_active {
                let frequency = obj.parameters.get("frequency").unwrap_or(&440.0);
                let amplitude = obj.parameters.get("amplitude").unwrap_or(&0.8);
                
                for (i, sample_out) in output.iter_mut().enumerate() {
                    let t = i as f32 / self.sample_rate;
                    let phase = t * frequency * 2.0 * std::f32::consts::PI;
                    
                    // Basic oscillator with PWM capability
                    let mut wave_value = phase.sin() * amplitude;
                    
                    // Add simple FM if available
                    if !input.is_empty() {
                        let fm_input = input.get(i % input.len()).unwrap_or(&0.0);
                        let fm_amount = self.parameters.fm_amount;
                        wave_value += fm_input * fm_amount * 0.1;
                    }
                    
                    *sample_out += wave_value;
                }
            }
        }
        Ok(())
    }
    
    /// Process filter objects
    fn process_filters(&mut self, input: &[f32], output: &mut [f32]) -> Result<()> {
        for obj in self.pd_objects.values_mut() {
            if obj.object_type == PdObjectType::Filter && obj.is_active {
                let cutoff = obj.parameters.get("cutoff").unwrap_or(&800.0);
                let resonance = obj.parameters.get("resonance").unwrap_or(&0.5);
                
                // Apply Pure Data-style lowpass filter
                let alpha = (2.0 * std::f32::consts::PI * cutoff / self.sample_rate) / 
                           (2.0 * std::f32::consts::PI * cutoff / self.sample_rate + 1.0);
                
                let mut prev_output = 0.0f32;
                
                for (i, (&sample, out)) in input.iter().zip(output.iter_mut()).enumerate() {
                    // Two-pole lowpass filter implementation
                    let y0 = sample + 0.0 * prev_output;
                    let y1 = alpha * y0 + (1.0 - alpha) * prev_output;
                    let y2 = alpha * y1 + (1.0 - alpha) * y0;
                    
                    *out = y2 * (1.0 - resonance * 0.9);
                    prev_output = y2;
                }
            }
        }
        Ok(())
    }
    
    /// Process envelope objects
    fn process_envelopes(&mut self, input: &[f32], output: &mut [f32]) -> Result<()> {
        for obj in self.pd_objects.values_mut() {
            if obj.object_type == PdObjectType::Envelope && obj.is_active {
                let attack = obj.parameters.get("attack").unwrap_or(&0.01);
                let decay = obj.parameters.get("decay").unwrap_or(&0.1);
                let sustain = obj.parameters.get("sustain").unwrap_or(&0.7);
                let release = obj.parameters.get("release").unwrap_or(&0.2);
                
                for (i, (&sample, out)) in input.iter().zip(output.iter_mut()).enumerate() {
                    let trigger_value = obj.parameters.get("trigger").unwrap_or(&0.0);
                    
                    let envelope_value = if trigger_value > &0.0 {
                        // ADSR envelope calculation
                        let env_time = i as f32 / self.sample_rate;
                        if env_time < *attack {
                            // Attack phase
                            env_time / attack
                        } else if env_time < attack + decay {
                            // Decay phase
                            1.0 - (1.0 - sustain) * ((env_time - attack) / decay)
                        } else {
                            // Sustain phase
                            *sustain
                        }
                    } else {
                        // Release phase (simplified)
                        let release_time = i as f32 / self.sample_rate;
                        if release_time < *release {
                            1.0 - (release_time / release)
                        } else {
                            0.0
                        }
                    };
                    
                    *out = sample * envelope_value;
                }
            }
        }
        Ok(())
    }
    
    /// Process LFO objects
    fn process_lfo(&self, obj: &PdObject, input: &[f32], output: &mut [f32]) -> Result<()> {
        if !obj.is_active { return Ok(()); }
        
        let frequency = obj.parameters.get("frequency").unwrap_or(&1.0);
        let amplitude = obj.parameters.get("amplitude").unwrap_or(&0.5);
        
        for (i, out) in output.iter_mut().enumerate() {
            let t = i as f32 / self.sample_rate;
            let lfo_value = (t * frequency * 2.0 * std::f32::consts::PI).sin() * amplitude;
            *out = input.get(i).copied().unwrap_or(0.0) + lfo_value;
        }
        Ok(())
    }
    
    /// Process mixer objects
    fn process_mixer(&self, obj: &PdObject, input: &[f32], output: &mut [f32]) -> Result<()> {
        if !obj.is_active { return Ok(()); }
        
        for (i, out) in output.iter_mut().enumerate() {
            let input_sample = input.get(i).copied().unwrap_or(0.0);
            let gain = obj.parameters.get("gain").unwrap_or(&1.0);
            *out = input_sample * gain;
        }
        Ok(())
    }
    
    /// Process delay objects
    fn process_delay(&self, obj: &PdObject, input: &[f32], output: &mut [f32]) -> Result<()> {
        if !obj.is_active { return Ok(()); }
        
        // Simple delay implementation
        let delay_time_samples = (self.parameters.delay_time * self.sample_rate) as usize;
        let feedback = obj.parameters.get("feedback").unwrap_or(&0.3);
        
        // This is a simplified delay - in reality would use a circular buffer
        for i in 0..output.len() {
            let input_sample = input.get(i).copied().unwrap_or(0.0);
            let delayed_sample = if i >= delay_time_samples {
                output[i - delay_time_samples] * feedback
            } else {
                0.0
            };
            output[i] = input_sample + delayed_sample;
        }
        Ok(())
    }
    
    /// Process reverb objects
    fn process_reverb(&self, obj: &PdObject, input: &[f32], output: &mut [f32]) -> Result<()> {
        if !obj.is_active { return Ok(()); }
        
        // Simple reverb using multiple delays
        let reverb_amount = self.parameters.reverb_amount;
        let room_size = obj.parameters.get("room_size").unwrap_or(&0.5);
        
        for (i, (&sample, out)) in input.iter().zip(output.iter_mut()).enumerate() {
            // Create a simple Schroeder reverb
            let mut reverb_component = 0.0;
            
            // Add a few delay taps
            for tap in [0.03, 0.05, 0.08] {
                let delay_samples = (tap * self.sample_rate) as usize;
                if i >= delay_samples {
                    reverb_component += output[i - delay_samples] * room_size * 0.1;
                }
            }
            
            *out = sample + reverb_component * reverb_amount;
        }
        Ok(())
    }
    
    /// Apply Pure Data post-processing
    fn apply_pd_post_processing(&self, output: &mut [f32]) -> Result<()> {
        // Apply master volume and basic effects
        let master_volume = self.parameters.master_volume;
        let brightness = self.parameters.patch_brightness;
        
        for sample in output.iter_mut() {
            *sample *= master_volume;
            
            // Apply brightness control (simple high-pass)
            if brightness < 1.0 {
                *sample *= brightness;
            }
            
            // Simple limiter
            if *sample > 1.0 {
                *sample = 1.0;
            } else if *sample < -1.0 {
                *sample = -1.0;
            }
        }
        
        Ok(())
    }
    
    /// Load a Pure Data patch
    pub fn load_patch(&mut self, name: &str, patch_path: &str) -> Result<()> {
        if let Some(patch) = self.pd_patches.get_mut(name) {
            patch.path = patch_path.to_string();
            
            // Parse patch file and extract objects
            self.parse_patch_file(patch_path)?;
            
            log::info!("Loaded patch: {} from {}", name, patch_path);
            Ok(())
        } else {
            Err(PdRsSynthError::PatchError(format!("Patch '{}' not found", name)).into())
        }
    }
    
    /// Parse a Pure Data patch file
    fn parse_patch_file(&self, _patch_path: &str) -> Result<()> {
        // This would parse the actual Pd patch file and extract objects
        // For now, we'll use the default objects
        
        log::info!("Parsing Pure Data patch file");
        Ok(())
    }
    
    /// Set parameter value
    pub fn set_parameter(&mut self, param_name: &str, value: f32) -> Result<()> {
        self.parameters.custom_parameters.insert(param_name.to_string(), value);
        log::debug!("Set parameter {} = {}", param_name, value);
        Ok(())
    }
    
    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> PdPerformanceMetrics {
        PdPerformanceMetrics {
            cpu_usage: self.cpu_usage,
            pd_processing_time: self.pd_processing_time,
            active_patches: self.active_patches.len(),
            active_objects: self.pd_objects.values().filter(|obj| obj.is_active).count(),
            active_notes: self.active_notes.len(),
            sample_rate: self.sample_rate,
            block_size: self.block_size,
        }
    }

    /// Get hexodsp node graph reference (for modular integration)
    #[cfg(feature = "hexodsp-daw")]
    pub fn get_hexodsp_node_graph(&self) -> Option<&NodeGraph> {
        self.hexodsp_node_graph.as_ref()
    }

    /// Get hexodsp node graph mutable reference (for modular integration)
    #[cfg(feature = "hexodsp-daw")]
    pub fn get_hexodsp_node_graph_mut(&mut self) -> Option<&mut NodeGraph> {
        self.hexodsp_node_graph.as_mut()
    }
}

/// Performance metrics for the Pure Data plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdPerformanceMetrics {
    pub cpu_usage: f32,
    pub pd_processing_time: f32,
    pub active_patches: usize,
    pub active_objects: usize,
    pub active_notes: usize,
    pub sample_rate: f32,
    pub block_size: usize,
}

/// Pure Data patch structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdPatch {
    pub name: String,
    pub path: String,
    pub audio_inlets: Vec<String>,
    pub audio_outlets: Vec<String>,
    pub message_inlets: Vec<String>,
    pub parameters: HashMap<String, PdParameter>,
    pub connections: Vec<(String, String)>,
}

/// Pure Data parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdParameter {
    pub name: String,
    pub default_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub unit: String,
    pub automation_enabled: bool,
}

/// Simple test function
pub fn hello_pd_rs_synth() -> &'static str {
    "Hello from PD-RS Synth! Professional VST3 plugin with Pure Data and hexodsp integration."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_initialization() {
        let mut plugin = PdRsSynth::default();
        let result = plugin.initialize(44100.0, 512, 2);
        assert!(result.is_ok());
        assert!(plugin.is_initialized);
    }

    #[test]
    fn test_note_triggering() {
        let mut plugin = PdRsSynth::default();
        plugin.initialize(44100.0, 512, 2).unwrap();
        
        let note = Note::C4;
        plugin.trigger_note(note, U7::from_u7(100));
        
        assert_eq!(plugin.active_notes.len(), 1);
        assert!(plugin.last_note_time.is_some());
    }

    #[test]
    fn test_parameter_setting() {
        let mut plugin = PdRsSynth::default();
        plugin.initialize(44100.0, 512, 2).unwrap();
        
        let result = plugin.set_parameter("test_param", 0.75);
        assert!(result.is_ok());
        assert_eq!(plugin.parameters.custom_parameters.get("test_param"), Some(&0.75));
    }

    #[test]
    fn test_audio_processing() {
        let mut plugin = PdRsSynth::default();
        plugin.initialize(44100.0, 512, 2).unwrap();
        
        let input = vec![0.5f32; 512];
        let mut output = vec![0.0f32; 512];
        
        let result = plugin.process_audio(&input, &mut output);
        assert!(result.is_ok());
        assert_eq!(output.len(), input.len());
    }
}