//! # Plugdata VST3 Plugin GUI
//!
//! Graphical user interface for the Plugdata Pure Data VST3 plugin.
//! Built using egui for immediate mode GUI rendering with Pure Data visual patching.

use egui::{CentralPanel, ComboBox, DragValue, Slider, TopBottomPanel};
use egui_extras::{self, TableBuilder};

/// Plugdata plugin GUI state
#[derive(Debug, Clone)]
pub struct PlugdataVst3Gui {
    /// Patch management
    pub current_patch_name: String,
    pub patch_list: Vec<String>,
    pub patch_loaded: bool,
    
    /// Pure Data canvas state
    pub canvas_zoom: f32,
    pub canvas_offset: egui::Vec2,
    pub selected_objects: Vec<String>,
    
    /// Audio controls
    pub master_volume: f32,
    pub audio_quality: f32,
    pub buffer_size: usize,
    pub sample_rate: u32,
    
    /// MIDI controls
    pub midi_channel: u8,
    pub velocity_curve: f32,
    pub note_range_min: u8,
    pub note_range_max: u8,
    
    /// Performance metrics
    pub cpu_usage: f32,
    pub pd_processing_time: f32,
    pub active_objects: usize,
    
    /// UI panels
    pub show_audio_settings: bool,
    pub show_midi_settings: bool,
    pub show_performance: bool,
}

impl Default for PlugdataVst3Gui {
    fn default() -> Self {
        Self {
            current_patch_name: "No Patch Loaded".to_string(),
            patch_list: vec![
                "Simple Synth".to_string(),
                "FM Radio".to_string(),
                "Drum Machine".to_string(),
                "Granular Synth".to_string(),
                "FM Complex".to_string(),
            ],
            patch_loaded: false,
            canvas_zoom: 1.0,
            canvas_offset: egui::Vec2::ZERO,
            selected_objects: Vec::new(),
            master_volume: 0.8,
            audio_quality: 0.8,
            buffer_size: 512,
            sample_rate: 44100,
            midi_channel: 0,
            velocity_curve: 1.0,
            note_range_min: 0,
            note_range_max: 127,
            cpu_usage: 0.0,
            pd_processing_time: 0.0,
            active_objects: 0,
            show_audio_settings: false,
            show_midi_settings: false,
            show_performance: false,
        }
    }
}

impl PlugdataVst3Gui {
    /// Render the main plugin GUI
    pub fn render(&mut self, ctx: &egui::Context) {
        self.render_top_panel(ctx);
        self.render_main_panel(ctx);
        self.render_bottom_panel(ctx);
    }
    
    /// Render the top panel with patch management
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("plugdata_top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.set_height(40.0);
                
                // Plugin name
                ui.label("PlugData VST3");
                ui.separator();
                
                // Patch information
                ui.label("Patch:");
                ui.label(egui::RichText::new(&self.current_patch_name).strong());
                
                // Patch status
                if self.patch_loaded {
                    ui.label(egui::RichText::new("LOADED").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("NO PATCH").color(egui::Color32::RED));
                }
                
                ui.separator();
                
                // Quick controls
                ui.label("Volume:");
                ui.add(Slider::new(&mut self.master_volume, 0.0..=1.0).show_value(false));
                ui.label(format!("{:.0}%", self.master_volume * 100.0));
                
                ui.separator();
                
                // Settings buttons
                if ui.button("🎵 Audio").clicked() {
                    self.show_audio_settings = !self.show_audio_settings;
                }
                
                if ui.button("🎹 MIDI").clicked() {
                    self.show_midi_settings = !self.show_midi_settings;
                }
                
                if ui.button("📊 Performance").clicked() {
                    self.show_performance = !self.show_performance;
                }
            });
            
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    self.create_new_patch();
                }
                
                if ui.button("Load").clicked() {
                    self.load_patch_dialog();
                }
                
                if ui.button("Save").clicked() {
                    self.save_patch();
                }
                
                if ui.button("Import Pd").clicked() {
                    self.import_pure_data_patch();
                }
                
                if ui.button("Export Pd").clicked() {
                    self.export_pure_data_patch();
                }
                
                ui.separator();
                
                ui.label("Zoom:");
                ui.add(DragValue::new(&mut self.canvas_zoom).clamp_range(0.1..=5.0).speed(0.1));
                ui.label(format!("{:.1}x", self.canvas_zoom));
                
                if ui.button("Reset View").clicked() {
                    self.canvas_zoom = 1.0;
                    self.canvas_offset = egui::Vec2::ZERO;
                }
            });
        });
    }
    
    /// Render the main Pure Data canvas
    fn render_main_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::click_and_drag(),
                );
                
                // Draw Pure Data canvas background
                self.draw_canvas_background(ui, rect);
                
                // Draw Pure Data objects
                if self.patch_loaded {
                    self.draw_pure_data_objects(ui, rect);
                } else {
                    self.draw_empty_canvas(ui, rect);
                }
            });
        });
    }
    
    /// Render the bottom panel with status and tools
    fn render_bottom_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom("plugdata_bottom_panel").show(ctx, |ui| {
            ui.set_height(60.0);
            
            egui::Grid::new("plugdata_status_grid")
                .spacing([10.0, 5.0])
                .show(ui, |ui| {
                    ui.label("Objects:");
                    ui.label(format!("{}", self.active_objects));
                    
                    ui.label("CPU:");
                    ui.colored_label(
                        if self.cpu_usage < 30.0 {
                            egui::Color32::GREEN
                        } else if self.cpu_usage < 60.0 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::RED
                        },
                        format!("{:.1}%", self.cpu_usage),
                    );
                    
                    ui.label("Pd Process:");
                    ui.label(format!("{:.2}ms", self.pd_processing_time));
                    
                    ui.label("Sample Rate:");
                    ui.label(format!("{} Hz", self.sample_rate));
                    
                    ui.label("Buffer:");
                    ui.label(format!("{} samples", self.buffer_size));
                });
        });
        
        // Settings panels (when toggled)
        if self.show_audio_settings {
            self.render_audio_settings(ctx);
        }
        
        if self.show_midi_settings {
            self.render_midi_settings(ctx);
        }
        
        if self.show_performance {
            self.render_performance_panel(ctx);
        }
    }
    
    /// Render audio settings panel
    fn render_audio_settings(&mut self, ctx: &egui::Context) {
        egui::Window::new("Audio Settings")
            .default_pos(egui::pos2(100.0, 100.0))
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Audio Processing");
                
                ui.vertical(|ui| {
                    ui.label("Sample Rate:");
                    ComboBox::from_label("")
                        .selected_text(format!("{} Hz", self.sample_rate))
                        .show_ui(ui, |ui| {
                            for rate in [44100, 48000, 88200, 96000] {
                                if ui.selectable_label(self.sample_rate == rate, format!("{} Hz", rate)).clicked() {
                                    self.sample_rate = rate;
                                }
                            }
                        });
                    
                    ui.label("Buffer Size:");
                    ComboBox::from_label("")
                        .selected_text(format!("{} samples", self.buffer_size))
                        .show_ui(ui, |ui| {
                            for size in [128, 256, 512, 1024, 2048] {
                                if ui.selectable_label(self.buffer_size == size, format!("{} samples", size)).clicked() {
                                    self.buffer_size = size;
                                }
                            }
                        });
                    
                    ui.label("Audio Quality:");
                    ui.add(Slider::new(&mut self.audio_quality, 0.0..=1.0));
                    ui.label(match self.audio_quality {
                        x if x < 0.3 => "Low",
                        x if x < 0.7 => "Medium", 
                        _ => "High"
                    });
                });
                
                if ui.button("Apply Changes").clicked() {
                    // Apply audio settings
                }
            });
    }
    
    /// Render MIDI settings panel
    fn render_midi_settings(&mut self, ctx: &egui::Context) {
        egui::Window::new("MIDI Settings")
            .default_pos(egui::pos2(100.0, 150.0))
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("MIDI Configuration");
                
                ui.vertical(|ui| {
                    ui.label("MIDI Channel:");
                    ui.add(DragValue::new(&mut self.midi_channel).clamp_range(1..=16));
                    
                    ui.label("Velocity Curve:");
                    ui.add(Slider::new(&mut self.velocity_curve, 0.1..=3.0).logarithmic(true));
                    ui.label(format!("{:.2}", self.velocity_curve));
                    
                    ui.label("Note Range:");
                    ui.horizontal(|ui| {
                        ui.label("Min:");
                        ui.add(DragValue::new(&mut self.note_range_min).clamp_range(0..=127));
                        ui.label("Max:");
                        ui.add(DragValue::new(&mut self.note_range_max).clamp_range(0..=127));
                    });
                });
                
                if ui.button("Test MIDI").clicked() {
                    // Test MIDI input
                }
            });
    }
    
    /// Render performance monitoring panel
    fn render_performance_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("Performance Monitor")
            .default_pos(egui::pos2(100.0, 200.0))
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Performance Metrics");
                
                egui::Grid::new("performance_grid")
                    .spacing([10.0, 5.0])
                    .show(ui, |ui| {
                        ui.label("Active Objects:");
                        ui.label(format!("{}", self.active_objects));
                        
                        ui.label("CPU Usage:");
                        ui.colored_label(
                            if self.cpu_usage < 30.0 {
                                egui::Color32::GREEN
                            } else if self.cpu_usage < 60.0 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::RED
                            },
                            format!("{:.1}%", self.cpu_usage),
                        );
                        
                        ui.label("Pd Processing:");
                        ui.label(format!("{:.2} ms", self.pd_processing_time));
                        
                        ui.label("Memory Usage:");
                        ui.label("~2.5 MB");
                        
                        ui.label("Audio Latency:");
                        ui.label(format!("{:.1} ms", (self.buffer_size as f32 / self.sample_rate as f32) * 1000.0));
                    });
                
                ui.separator();
                
                if ui.button("Clear Statistics").clicked() {
                    self.cpu_usage = 0.0;
                    self.pd_processing_time = 0.0;
                }
            });
    }
    
    /// Draw the canvas background
    fn draw_canvas_background(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let painter = ui.painter();
        
        // Canvas background
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
        
        // Grid
        let grid_size = 20.0;
        for x in (rect.left() as i32..(rect.right() as i32)).step_by(grid_size as usize) {
            painter.line_segment(
                [egui::pos2(x as f32, rect.top()), egui::pos2(x as f32, rect.bottom())],
                egui::Stroke::new(1.0, egui::Color32::from_gray(35)),
            );
        }
        
        for y in (rect.top() as i32..(rect.bottom() as i32)).step_by(grid_size as usize) {
            painter.line_segment(
                [egui::pos2(rect.left(), y as f32), egui::pos2(rect.right(), y as f32)],
                egui::Stroke::new(1.0, egui::Color32::from_gray(35)),
            );
        }
    }
    
    /// Draw Pure Data objects on canvas
    fn draw_pure_data_objects(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        // Draw some example Pure Data objects
        let center = rect.center();
        
        // Oscillator object
        self.draw_pd_object(ui, center + egui::vec2(-100, 0), "osc~ 440");
        
        // Filter object
        self.draw_pd_object(ui, center, "lowpass 1000");
        
        // Envelope object
        self.draw_pd_object(ui, center + egui::vec2(100, 0), "adsr~ 0.01 0.1 0.7 0.2");
        
        // Amplifier object
        self.draw_pd_object(ui, center + egui::vec2(0, 100), "*~ 0.5");
        
        // Connections
        self.draw_pd_connection(ui, center + egui::vec2(-70, 0), center + egui::vec2(-30, 0));
        self.draw_pd_connection(ui, center + egui::vec2(70, 0), center + egui::vec2(30, 0));
        self.draw_pd_connection(ui, center + egui::vec2(0, 30), center + egui::vec2(0, 70));
    }
    
    /// Draw empty canvas state
    fn draw_empty_canvas(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No Pure Data Patch Loaded\n\nLoad a .pd file to start patching",
            egui::FontId::proportional(16.0),
            egui::Color32::GRAY,
        );
    }
    
    /// Draw a Pure Data object box
    fn draw_pd_object(&self, ui: &mut egui::Ui, pos: egui::Pos2, label: &str) {
        let painter = ui.painter();
        let size = egui::vec2(120.0, 30.0);
        let rect = egui::Rect::from_min_size(pos, size);
        
        // Object background
        painter.rect_filled(
            rect,
            egui::Rounding::same(3.0),
            egui::Color32::from_rgb(200, 200, 250),
        );
        
        // Object border
        painter.rect_stroke(
            rect,
            egui::Rounding::same(3.0),
            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 200)),
        );
        
        // Inlet dots
        for i in 0..2 {
            let inlet_pos = egui::pos2(rect.left() - 5.0, rect.top() + 10.0 + i as f32 * 15.0);
            painter.circle_filled(inlet_pos, 3.0, egui::Color32::BLACK);
        }
        
        // Outlet dots  
        for i in 0..2 {
            let outlet_pos = egui::pos2(rect.right() + 5.0, rect.top() + 10.0 + i as f32 * 15.0);
            painter.circle_filled(outlet_pos, 3.0, egui::Color32::BLACK);
        }
        
        // Object label
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::monospace(10.0),
            egui::Color32::BLACK,
        );
    }
    
    /// Draw connection between Pure Data objects
    fn draw_pd_connection(&self, ui: &mut egui::Ui, from: egui::Pos2, to: egui::Pos2) {
        let painter = ui.painter();
        
        // Calculate control points for curved connection
        let mid_x = (from.x + to.x) / 2.0;
        let control_point1 = egui::pos2(mid_x, from.y);
        let control_point2 = egui::pos2(mid_x, to.y);
        
        // Draw curved line
        let path = egui::epaint::QuadraticBezierCurve::from_points(from, control_point1, control_point2);
        painter.add(egui::Shape::QuadraticBezierCurve {
            points: [from, control_point1, control_point2],
            stroke: egui::Stroke::new(2.0, egui::Color32::BLACK),
            fill: egui::Color32::TRANSPARENT,
        });
        
        // Arrow head
        let direction = (to - from).normalize();
        let arrow_start = to - direction * 10.0;
        let arrow_left = arrow_start + egui::vec2(-5.0, -5.0);
        let arrow_right = arrow_start + egui::vec2(-5.0, 5.0);
        
        painter.triangle_filled(to, arrow_left, arrow_right, egui::Color32::BLACK);
    }
    
    // Stub methods for patch management
    fn create_new_patch(&mut self) {
        self.current_patch_name = "Untitled".to_string();
        self.patch_loaded = false;
    }
    
    fn load_patch_dialog(&mut self) {
        // Open file dialog to load patch
    }
    
    fn save_patch(&mut self) {
        // Save current patch
    }
    
    fn import_pure_data_patch(&mut self) {
        // Import .pd file
        self.patch_loaded = true;
        self.active_objects = 15; // Example value
    }
    
    fn export_pure_data_patch(&mut self) {
        // Export to .pd file
    }
    
    /// Update performance metrics
    pub fn update_metrics(&mut self, cpu_usage: f32, pd_time: f32, objects: usize) {
        self.cpu_usage = cpu_usage;
        self.pd_processing_time = pd_time;
        self.active_objects = objects;
    }
}