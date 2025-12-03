use eframe::egui;
use chrono;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::sync::Arc;
use crate::core::{VideoAction, ActionConfig};
use crate::actions::*;

// Message types for communication between threads
enum AppMessage {
    Log(String),
    Progress(f32),
    Finished,
    Error(String),
}

// App State
struct VideoMatrixApp {
    input_dir: String,
    output_dir: String,
    selected_actions: Vec<String>,
    is_processing: bool,
    progress: f32,
    log_messages: Vec<String>,
    
    // Thread communication
    rx: Option<Receiver<AppMessage>>,
    
    // Tab State
    current_tab: Tab,
    
    // Checkbox State
    checkboxes: Vec<(String, String, bool)>, // (Display Name, ID, Checked)
}

// Tab Enum
#[derive(PartialEq, Clone, Copy)]
enum Tab {
    All,       // All-in-One Panel
    Additional, // Additional Features
}

impl Default for Tab {
    fn default() -> Self {
        Tab::All
    }
}

impl Default for VideoMatrixApp {
    fn default() -> Self {
        // Initialize all checkboxes (English version)
        let mut checkboxes = Vec::new();
        
        // === All-in-One Panel (Tab::All) ===
        // Basic Editing & Parameters
        checkboxes.extend(vec![
            ("One-Click MD5 (Remux)".to_string(), "md5".to_string(), false),
            ("Random Micro-Crop (1-5%)".to_string(), "crop".to_string(), false),
            ("Trim Head/Tail (1s each)".to_string(), "cut_head_tail".to_string(), false),
            ("Micro Rotation (±1.5°)".to_string(), "rotate".to_string(), false),
            ("Non-linear Speed (0.9-1.1x)".to_string(), "speed".to_string(), false),
            ("Mirror Flip".to_string(), "mirror".to_string(), false),
            ("Force 60 FPS".to_string(), "fps_60".to_string(), false),
            ("High Bitrate (15Mbps)".to_string(), "bitrate_hq".to_string(), false),
        ]);
        
        // Visual Enhancements
        checkboxes.extend(vec![
            ("Smart Sharpen".to_string(), "sharpen".to_string(), false),
            ("Smart Sharpen (Portrait)".to_string(), "portrait".to_string(), false),
            ("Smart Denoise".to_string(), "denoise".to_string(), false),
            ("Smart Denoise (Clean)".to_string(), "clean".to_string(), false),
            ("Film Grain".to_string(), "grain".to_string(), false),
            ("Smart Soft Focus".to_string(), "blur".to_string(), false),
            ("Random Color Temp".to_string(), "color".to_string(), false),
            ("Cinematic Vignette".to_string(), "vignette".to_string(), false),
            ("B&W Nostalgia".to_string(), "bw".to_string(), false),
            ("Smart Border Fill".to_string(), "border".to_string(), false),
            ("Smart Frame Pull".to_string(), "pull".to_string(), false),
            ("Corner Blur Mask".to_string(), "corner".to_string(), false),
        ]);
        
        // AI & AB Modes
        checkboxes.extend(vec![
            ("AI Random Zoom (ZoomPan)".to_string(), "zoom".to_string(), false),
            ("AI Move Dissolve".to_string(), "dissolve".to_string(), false),
            ("AI Random Light Scan".to_string(), "scan".to_string(), false),
            ("Bounce Effect".to_string(), "bounce".to_string(), false),
            ("Trifold Effect".to_string(), "trifold".to_string(), false),
            ("Lava AB Mode".to_string(), "lava".to_string(), false),
            ("3D Flash".to_string(), "flash".to_string(), false),
            ("Progressive Process".to_string(), "progressive".to_string(), false),
            ("AB Blend Mode".to_string(), "ab_blend".to_string(), false),
            ("AB Glitch Effect".to_string(), "ab_glitch".to_string(), false),
            ("AB Shake Effect".to_string(), "ab_shake".to_string(), false),
            ("AB Chroma Offset".to_string(), "ab_chroma".to_string(), false),
            ("AB Video Replace".to_string(), "ab_replace".to_string(), false),
            ("Advanced AB Replace".to_string(), "ab_advanced_replace".to_string(), false),
        ]);
        
        // Audio & Others
        checkboxes.extend(vec![
            ("Mute Video".to_string(), "mute".to_string(), false),
            ("Mix Weak White Noise".to_string(), "audio_noise".to_string(), false),
            ("Audio Pitch Shift".to_string(), "pitch".to_string(), false),
            ("Modify Timestamp Only".to_string(), "touch".to_string(), false),
        ]);
        
        // === Additional Features (Tab::Additional) ===
        // Strong Deduplication
        checkboxes.extend(vec![
            ("Strong Crop (8-12%)".to_string(), "strong_crop".to_string(), false),
            ("Add Watermark".to_string(), "watermark".to_string(), false),
            ("Modify Encode Params".to_string(), "encode".to_string(), false),
            ("Add Sticker".to_string(), "sticker".to_string(), false),
            ("Mask Overlay".to_string(), "mask".to_string(), false),
            ("Real AB Replace".to_string(), "ab_real_replace".to_string(), false),
        ]);
        
        // OpenCV Features
        checkboxes.extend(vec![
            ("Face Detection".to_string(), "face_detection".to_string(), false),
            ("Object Tracking".to_string(), "object_tracking".to_string(), false),
            ("OpenCV Filter".to_string(), "opencv_filter".to_string(), false),
        ]);
        
        // New Material Features
        checkboxes.extend(vec![
            ("Light Effect Overlay".to_string(), "light_effect".to_string(), false),
            ("Picture-in-Picture".to_string(), "pip".to_string(), false),
            ("Edge Effect".to_string(), "edge_effect".to_string(), false),
            ("Goods Template".to_string(), "goods_template".to_string(), false),
        ]);
        
        Self {
            input_dir: String::new(),
            output_dir: String::new(),
            selected_actions: Vec::new(),
            is_processing: false,
            progress: 0.0,
            current_tab: Tab::All,
            rx: None,
            log_messages: vec![
                "✨ Video Matrix Pro Ready".to_string(),
                "💡 Tip: Select input folder, check features, then click 'Start Processing'".to_string(),
            ],
            checkboxes,
        }
    }
}

impl eframe::App for VideoMatrixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for messages from the processing thread
        if let Some(rx) = self.rx.take() {
            let mut keep_rx = true;
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    AppMessage::Log(text) => self.log_internal(text),
                    AppMessage::Progress(p) => self.progress = p,
                    AppMessage::Finished => {
                        self.is_processing = false;
                        keep_rx = false;
                        self.log_internal("🎉 All tasks completed!".to_string());
                        self.progress = 1.0;
                    },
                    AppMessage::Error(e) => {
                        self.log_internal(format!("❌ Error: {}", e));
                        self.is_processing = false;
                        keep_rx = false;
                    }
                }
            }
            if keep_rx {
                self.rx = Some(rx);
            }
        }

        // Bottom Panel for Controls, Progress, and Logs
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .min_height(150.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(5.0);
                    
                    // Control Area
                    ui.horizontal(|ui| {
                        ui.label(format!("Selected {} features", self.selected_actions.len()));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🛑 Stop").clicked() {
                                self.stop_processing();
                            }
                            
                            let can_start = !self.input_dir.is_empty() && !self.selected_actions.is_empty() && !self.is_processing;
                            let start_btn = egui::Button::new("🚀 Start Processing");
                            
                            // Status Text
                            if !can_start {
                                if self.input_dir.is_empty() {
                                    ui.colored_label(egui::Color32::RED, "⚠️ Select Input Folder");
                                } else if self.selected_actions.is_empty() {
                                    ui.colored_label(egui::Color32::RED, "⚠️ Select Features");
                                } else if self.is_processing {
                                    ui.colored_label(egui::Color32::YELLOW, "⏳ Processing...");
                                }
                            }

                            if can_start {
                                if ui.add(start_btn).clicked() {
                                    self.start_processing();
                                }
                            } else {
                                let response = ui.add_enabled(false, start_btn);
                                if self.input_dir.is_empty() {
                                    response.on_disabled_hover_text("Please select an input directory");
                                } else if self.selected_actions.is_empty() {
                                    response.on_disabled_hover_text("Please select at least one feature");
                                } else if self.is_processing {
                                    response.on_disabled_hover_text("Processing is already in progress");
                                }
                            }
                        });
                    });
                    
                    ui.add_space(5.0);
                    
                    // Progress Bar
                    ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                    
                    ui.add_space(5.0);
                    ui.separator();
                    
                    // Log Area
                    ui.collapsing("📋 Processing Logs", |ui| {
                        let text_style = egui::TextStyle::Body;
                        let row_height = ui.text_style_height(&text_style);
                        let total_rows = self.log_messages.len();
                        
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                                for row in row_range {
                                    ui.label(&self.log_messages[row]);
                                }
                            });
                    });
                    ui.add_space(5.0);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.heading("Video Matrix Pro V5.4");
            ui.colored_label(egui::Color32::GRAY, "Rust Refactored - High Performance Video Tool");
            ui.separator();
            
            // Workspace
            ui.collapsing("📁 Workspace", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Input:");
                    let input_response = ui.add(
                        egui::TextEdit::singleline(&mut self.input_dir)
                            .hint_text("Drag folder here...")
                    );
                    if ui.button("Browse").clicked() {
                        // Use rfd to open folder dialog
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.input_dir = path.to_string_lossy().to_string();
                            self.log(&format!("Input directory selected: {}", self.input_dir));
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Output:");
                    let output_response = ui.add(
                        egui::TextEdit::singleline(&mut self.output_dir)
                            .hint_text("Output path (optional, default: input/output)...")
                    );
                    if ui.button("Save To").clicked() {
                        // Use rfd to open folder dialog
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.output_dir = path.to_string_lossy().to_string();
                            self.log(&format!("Output directory selected: {}", self.output_dir));
                        }
                    }
                });
            });
            
            ui.separator();
            
            // Tab Selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::All, "All-in-One Panel");
                ui.selectable_value(&mut self.current_tab, Tab::Additional, "Additional Features");
            });
            
            ui.separator();
            
            // Scrollable Area for Features
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Collect updates
                let mut updates = Vec::new();
                
                // Show features based on current tab
                match self.current_tab {
                    Tab::All => {
                        // All-in-One Panel
                        ui.heading("✂️ Basic Editing & Parameters");
                        for i in 0..8 {
                            let (name, id, checked) = &mut self.checkboxes[i];
                            let old_checked = *checked;
                            if ui.checkbox(checked, name.as_str()).changed() {
                                updates.push((id.clone(), name.clone(), old_checked, *checked));
                            }
                        }
                        
                        ui.separator();
                        ui.heading("🎨 Visual Enhancements");
                        for i in 8..20 {
                            let (name, id, checked) = &mut self.checkboxes[i];
                            let old_checked = *checked;
                            if ui.checkbox(checked, name.as_str()).changed() {
                                updates.push((id.clone(), name.clone(), old_checked, *checked));
                            }
                        }
                        
                        ui.separator();
                        ui.heading("🤖 AI & AB Modes");
                        for i in 20..34 {
                            let (name, id, checked) = &mut self.checkboxes[i];
                            let old_checked = *checked;
                            if ui.checkbox(checked, name.as_str()).changed() {
                                updates.push((id.clone(), name.clone(), old_checked, *checked));
                            }
                        }
                        
                        ui.separator();
                        ui.heading("🎵 Audio & Others");
                        for i in 34..38 {
                            let (name, id, checked) = &mut self.checkboxes[i];
                            let old_checked = *checked;
                            if ui.checkbox(checked, name.as_str()).changed() {
                                updates.push((id.clone(), name.clone(), old_checked, *checked));
                            }
                        }
                    }
                    Tab::Additional => {
                        // Additional Features
                        ui.heading("💪 Strong Deduplication");
                        for i in 38..44 {
                            let (name, id, checked) = &mut self.checkboxes[i];
                            let old_checked = *checked;
                            if ui.checkbox(checked, name.as_str()).changed() {
                                updates.push((id.clone(), name.clone(), old_checked, *checked));
                            }
                        }
                        
                        ui.separator();
                        ui.heading("👁️ OpenCV Features");
                        for i in 44..47 {
                            let (name, id, checked) = &mut self.checkboxes[i];
                            let old_checked = *checked;
                            if ui.checkbox(checked, name.as_str()).changed() {
                                updates.push((id.clone(), name.clone(), old_checked, *checked));
                            }
                        }
                        
                        ui.separator();
                        ui.heading("✨ New Material Features");
                        for i in 47..51 {
                            let (name, id, checked) = &mut self.checkboxes[i];
                            let old_checked = *checked;
                            if ui.checkbox(checked, name.as_str()).changed() {
                                updates.push((id.clone(), name.clone(), old_checked, *checked));
                            }
                        }
                    }
                }
                
                // Process updates
                for (id, name, _old_checked, new_checked) in updates {
                    if new_checked {
                        self.selected_actions.push(id);
                    } else {
                        self.selected_actions.retain(|x| x != &id);
                    }
                    self.log(&format!("{} {}", if new_checked { "Selected" } else { "Unselected" }, name));
                }
            });
        });
        
        // Request repaint to keep UI responsive during processing
        if self.is_processing {
            ctx.request_repaint();
        }
    }
}

impl VideoMatrixApp {
    fn log(&mut self, message: &str) {
        self.log_internal(message.to_string());
    }

    fn log_internal(&mut self, message: String) {
        let timestamp = chrono::Local::now().format("[%H:%M:%S]").to_string();
        self.log_messages.push(format!("{} {}", timestamp, message));
        // Limit log size
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
    }
    
    fn start_processing(&mut self) {
        self.is_processing = true;
        self.progress = 0.0;
        self.log("🚀 Starting processing (Background Thread)...");
        
        let input_dir = self.input_dir.clone();
        let output_dir = if self.output_dir.is_empty() {
            format!("{}/output", self.input_dir)
        } else {
            self.output_dir.clone()
        };
        let selected_actions = self.selected_actions.clone();
        
        // Create channel
        let (tx, rx) = channel();
        self.rx = Some(rx);
        
        // Clone for thread
        let tx_clone = tx.clone();
        
        // Spawn thread
        thread::spawn(move || {
            if let Err(e) = Self::process_thread(input_dir, output_dir, selected_actions, tx_clone) {
                // We can't easily send the error back if the channel is closed, but we try
                // In a real app we might want better error handling
                eprintln!("Thread error: {}", e);
            }
        });
    }

    fn process_thread(input_dir: String, output_dir: String, actions: Vec<String>, tx: Sender<AppMessage>) -> anyhow::Result<()> {
        let _ = tx.send(AppMessage::Log(format!("📂 Input: {}", input_dir)));
        let _ = tx.send(AppMessage::Log(format!("📂 Output: {}", output_dir)));
        let _ = tx.send(AppMessage::Log(format!("✅ Selected {} features", actions.len())));
        
        // Scan video files
        let _ = tx.send(AppMessage::Log("🔍 Scanning for video files...".to_string()));
        let video_files = Self::scan_video_files_static(&input_dir);
        
        if video_files.is_empty() {
            let _ = tx.send(AppMessage::Error("No video files found".to_string()));
            return Ok(());
        }
        
        let _ = tx.send(AppMessage::Log(format!("📹 Found {} video files", video_files.len())));
        
        let total_tasks = (video_files.len() * actions.len()) as f32;
        let mut completed_tasks = 0.0;
        
        // Create output directory
        let out_path = PathBuf::from(&output_dir);
        if let Err(e) = fs::create_dir_all(&out_path) {
            let _ = tx.send(AppMessage::Error(format!("Failed to create output directory: {}", e)));
            return Ok(());
        }
        
        // Process each video file
        let config = ActionConfig::default();
        for video_file in &video_files {
            let video_path = Path::new(video_file);
            let filename = video_path.file_name().unwrap().to_string_lossy();
            
            for action_id in &actions {
                let _ = tx.send(AppMessage::Log(format!("  ⏳ Processing: {} [{}]...", filename, action_id)));
                
                // Call corresponding action
                let result = Self::execute_action_static(action_id, video_path, &out_path, &config);
                
                match result {
                    Ok(_) => {
                        completed_tasks += 1.0;
                        let _ = tx.send(AppMessage::Progress(completed_tasks / total_tasks));
                        let _ = tx.send(AppMessage::Log(format!("  ✅ {} Completed ({})", action_id, filename)));
                    }
                    Err(e) => {
                        let _ = tx.send(AppMessage::Log(format!("  ❌ {} Failed ({}): {}", action_id, filename, e)));
                    }
                }
            }
        }
        
        let _ = tx.send(AppMessage::Finished);
        Ok(())
    }
    
    fn scan_video_files(&self, dir: &str) -> Vec<String> {
        Self::scan_video_files_static(dir)
    }

    fn scan_video_files_static(dir: &str) -> Vec<String> {
        let mut video_files = Vec::new();
        let video_extensions = vec!["mp4", "mov", "mkv", "avi", "wmv", "flv", "webm", "m4v"];
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(ext) = entry.path().extension() {
                            if let Some(ext_str) = ext.to_str() {
                                if video_extensions.contains(&ext_str.to_lowercase().as_str()) {
                                    if let Some(path_str) = entry.path().to_str() {
                                        video_files.push(path_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        video_files
    }
    
    fn execute_action(&self, action_id: &str, src: &Path, out_dir: &Path, config: &ActionConfig) -> anyhow::Result<()> {
        Self::execute_action_static(action_id, src, out_dir, config)
    }

    fn execute_action_static(action_id: &str, src: &Path, out_dir: &Path, config: &ActionConfig) -> anyhow::Result<()> {
        // Call corresponding action function based on action_id
        match action_id {
            "md5" => Md5Action.execute(src, out_dir, config),
            "crop" => CropAction.execute(src, out_dir, config),
            "cut_head_tail" => CutAction.execute(src, out_dir, config),
            "rotate" => RotateAction.execute(src, out_dir, config),
            "speed" => SpeedAction.execute(src, out_dir, config),
            "mirror" => MirrorAction.execute(src, out_dir, config),
            "fps_60" => FpsAction.execute(src, out_dir, config),
            "bitrate_hq" => BitrateAction.execute(src, out_dir, config),
            "sharpen" => SharpenAction.execute(src, out_dir, config),
            "portrait" => PortraitAction.execute(src, out_dir, config),
            "denoise" => DenoiseAction.execute(src, out_dir, config),
            "clean" => CleanAction.execute(src, out_dir, config),
            "grain" => GrainAction.execute(src, out_dir, config),
            "blur" => BlurAction.execute(src, out_dir, config),
            "color" => ColorAction.execute(src, out_dir, config),
            "vignette" => VignetteAction.execute(src, out_dir, config),
            "bw" => BwAction.execute(src, out_dir, config),
            "border" => BorderAction.execute(src, out_dir, config),
            "pull" => PullAction.execute(src, out_dir, config),
            "corner" => CornerAction.execute(src, out_dir, config),
            "zoom" => ZoomAction.execute(src, out_dir, config),
            "dissolve" => DissolveAction.execute(src, out_dir, config),
            "scan" => ScanAction.execute(src, out_dir, config),
            "bounce" => BounceAction.execute(src, out_dir, config),
            "trifold" => TrifoldAction.execute(src, out_dir, config),
            "lava" => LavaAction.execute(src, out_dir, config),
            "flash" => FlashAction.execute(src, out_dir, config),
            "progressive" => ProgressiveAction.execute(src, out_dir, config),
            "ab_blend" => AbBlendAction.execute(src, out_dir, config),
            "ab_glitch" => AbGlitchAction.execute(src, out_dir, config),
            "ab_shake" => AbShakeAction.execute(src, out_dir, config),
            "ab_chroma" => AbChromaAction.execute(src, out_dir, config),
            "ab_replace" => AbReplaceAction.execute(src, out_dir, config),
            "ab_advanced_replace" => AbAdvancedReplaceAction.execute(src, out_dir, config),
            "mute" => MuteAction.execute(src, out_dir, config),
            "audio_noise" => AudioNoiseAction.execute(src, out_dir, config),
            "pitch" => PitchAction.execute(src, out_dir, config),
            "touch" => TouchAction.execute(src, out_dir, config),
            "strong_crop" => StrongCropAction.execute(src, out_dir, config),
            "watermark" => WatermarkAction.execute(src, out_dir, config),
            "encode" => EncodeAction.execute(src, out_dir, config),
            "ab_real_replace" => AbRealReplaceAction.execute(src, out_dir, config),
            "sticker" => StickerAction.execute(src, out_dir, config),
            "mask" => MaskAction.execute(src, out_dir, config),
            "face_detection" => FaceDetectionAction.execute(src, out_dir, config),
            "object_tracking" => ObjectTrackingAction.execute(src, out_dir, config),
            "opencv_filter" => OpencvFilterAction.execute(src, out_dir, config),
            "light_effect" => LightEffectAction.execute(src, out_dir, config),
            "pip" => PipAction.execute(src, out_dir, config),
            "edge_effect" => EdgeEffectAction.execute(src, out_dir, config),
            "goods_template" => GoodsTemplateAction.execute(src, out_dir, config),
            _ => Err(anyhow::anyhow!("Unknown action: {}", action_id)),
        }
    }
    
    fn stop_processing(&mut self) {
        if self.is_processing {
            self.is_processing = false;
            self.rx = None; // Detach receiver
            self.log("🛑 User stopped processing");
        }
    }
}

// Main function
pub fn run_desktop_app() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("Video Matrix Pro V5.4 (Rust Desktop)"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Video Matrix Pro",
        options,
        Box::new(|_cc| Ok(Box::<VideoMatrixApp>::default())),
    )
}