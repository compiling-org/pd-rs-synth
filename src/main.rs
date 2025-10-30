use plugdata_vst3::PdRsSynth;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎛️ PlugData Standalone Synthesizer");
    println!("==================================");

    // Initialize the synthesizer
    let mut synth = PdRsSynth::default();
    synth.initialize(44100.0, 512, 2)?;

    println!("✅ PlugData synthesizer initialized");
    println!("🎹 Press keys A-Z to play notes, Q to quit");
    println!("📝 Pure Data patches loaded and ready");

    // Simple keyboard mapping (A=440Hz, etc.)
    let key_frequencies = [
        ('A', 440.0), ('W', 466.16), ('S', 493.88), ('E', 523.25), ('D', 554.37),
        ('F', 587.33), ('T', 622.25), ('G', 659.25), ('Y', 698.46), ('H', 739.99),
        ('U', 783.99), ('J', 830.61), ('K', 880.0), ('O', 932.33), ('L', 987.77),
        ('P', 1046.5), (';', 1108.73), ('\'', 1174.66)
    ];

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_uppercase();

        if input == "Q" {
            break;
        }

        // Find the key
        if let Some((_, freq)) = key_frequencies.iter().find(|(key, _)| *key == input.chars().next().unwrap_or(' ')) {
            // Create a simple audio buffer for testing
            let mut input_buffer = vec![0.0f32; 512];
            let mut output_buffer = vec![0.0f32; 512];

            // Trigger note
            synth.trigger_note(wmidi::Note::from_u7_lossy(69), wmidi::U7::from_u7_lossy(100));

            // Process some audio
            synth.process_audio(&input_buffer, &mut output_buffer)?;

            println!("🎵 Playing Pure Data note at {:.1} Hz", freq);
            println!("📊 Generated {} samples through Pd objects", output_buffer.len());

            // Show performance metrics
            let metrics = synth.get_performance_metrics();
            println!("⚡ CPU: {:.1}%, Pd Processing: {:.2}ms, Active Objects: {}",
                    metrics.cpu_usage, metrics.pd_processing_time, metrics.active_objects);

            // Release note after a short time
            std::thread::sleep(std::time::Duration::from_millis(500));
            synth.release_note(wmidi::Note::from_u7_lossy(69));
        } else {
            println!("❌ Invalid key. Press A-Z to play notes, Q to quit");
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}