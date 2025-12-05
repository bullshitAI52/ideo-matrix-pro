use eframe::egui;
use chrono;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::sync::Arc;
use crate::core::{VideoAction, ActionConfig};
use crate::core::ai::{AIService, AIResponse};
use crate::actions::*;
use rayon::prelude::*;

// Message types for communication between threads
enum AppMessage {
    Log(String),
    Progress(f32),
    Finished,
    Error(String),
    AIResult(AIResponse),
    AIConnectionResult(String),
}

// App State
struct VideoMatrixApp {
    input_dir: String,
    output_dir: String,
    selected_actions: Vec<String>,
    is_processing: bool,
    progress: f32,
    log_messages: Vec<String>,
    
    // Material Paths
    watermark_path: String,
    mask_path: String,
    sticker_path: String,
    border_path: String,
    light_effect_path: String,
    pip_path: String,
    goods_path: String,
    mask_video_path: String,
    
    // Thread communication
    rx: Option<Receiver<AppMessage>>,
    runtime: Arc<tokio::runtime::Runtime>,
    
    // Tab State
    current_tab: Tab,
    
    // Checkbox State
    checkboxes: Vec<(String, String, bool)>, // (Display Name, ID, Checked)
    
    // Action Parameters
    action_params: std::collections::HashMap<String, serde_json::Value>,
    
    // Settings Dialog State
    show_settings_dialog: bool,
    settings_action_id: String,
    // Crop parameters
    crop_min: f32,
    crop_max: f32,
    // Watermark parameters
    watermark_position: String,
    watermark_opacity: f32,
    
    // --- New Parameters ---
    // Basic
    rotate_angle: f32,      // Max rotation angle (degrees)
    speed_range: f32,       // Speed variation (e.g. 0.1 for ±10%)
    target_fps: u32,        // Target FPS (30, 60)
    target_bitrate: String, // e.g. "10M", "15M"
    
    // Visual
    sharpen_strength: f32,  // 0.0 - 5.0
    denoise_strength: f32,  // 0.0 - 20.0 (h value)
    blur_strength: f32,     // sigma
    grain_strength: f32,    // 0.0 - 0.5
    vignette_strength: f32, // angle/range
    
    // Effects
    border_width: i32,      // pixels for blur border
    
    // --- Additional Parameters ---
    // Basic editing
    cut_seconds: f32,           // seconds to cut from start/end
    mirror_direction: String,   // "horizontal", "vertical", "both"
    strong_crop_ratio: f32,     // crop ratio for strong crop
    
    // Visual enhancements
    portrait_strength: f32,     // portrait blur strength
    color_temp_range: i32,      // color temperature adjustment range
    pull_width: i32,            // border width for pull effect
    progressive_ratio: f32,     // frame drop ratio
    corner_radius: f32,         // corner blur radius
    
    // AI & Effects
    zoom_range: f32,            // zoom scale range
    dissolve_strength: f32,     // dissolve effect strength
    scan_strength: f32,         // light scan strength
    bounce_amplitude: f32,      // bounce effect amplitude
    trifold_spacing: i32,       // trifold spacing in pixels
    flash_strength: f32,        // 3D flash strength
    lava_strength: f32,         // lava AB mode strength
    
    // Audio
    noise_strength: f32,        // white noise volume
    pitch_range: f32,           // pitch shift range in semitones
    
    // AI Deduplication
    deepseek_api_key: String,   // DeepSeek API key
    deepseek_base_url: String,  // API base URL
    ai_prompt: String,          // User's AI processing request
    
    // Mask Video
    mask_video_opacity: f32,    // mask video opacity (0.0-1.0)
    mask_video_blend_mode: String, // blend mode (multiply/screen/overlay/add)
    mask_video_scale: String,   // scale mode (stretch/crop/fit)
    
    // 单个视频功能叠加模式
    single_video_mode: bool,    // true: 所有功能叠加到单个视频; false: 每个功能生成独立视频

    // UI Customization
    show_ui_settings: bool,
    ui_font_scale: f32,
    ui_bg_color: [u8; 3],       // RGB
    ui_bg_alpha: u8,            // Alpha 0-255
}

// Tab Enum
#[derive(PartialEq, Clone, Copy)]
enum Tab {
    All,       // All-in-One Panel
    Additional, // Additional Features
    Materials,  // New Materials Tab
    Help,      // Help & Documentation
    AIDedup,   // AI-powered deduplication
    ProcessingMode, // 处理模式设置
    Presets,   // Configuration presets
    Preview,   // Effect preview
}

impl Default for Tab {
    fn default() -> Self {
        Tab::All
    }
}

impl Default for VideoMatrixApp {
    fn default() -> Self {
        // Initialize all checkboxes (中文版本)
        let mut checkboxes = Vec::new();
        
        // === All-in-One Panel (Tab::All) ===
        // 基础编辑与参数
        checkboxes.extend(vec![
            ("一键MD5 (Remux)".to_string(), "md5".to_string(), false),
            ("随机微裁剪 (1-5%)".to_string(), "crop".to_string(), false),
            ("首尾去秒 (各1秒)".to_string(), "cut_head_tail".to_string(), false),
            ("微旋转 (±1.5°)".to_string(), "rotate".to_string(), false),
            ("非线性变速 (0.95-1.05x)".to_string(), "speed".to_string(), false),
            ("镜像翻转".to_string(), "mirror".to_string(), false),
            ("强制60帧".to_string(), "fps_60".to_string(), false),
            ("高码率 (15Mbps)".to_string(), "bitrate_hq".to_string(), false),
        ]);
        
        // 视觉增强
        checkboxes.extend(vec![
            ("智能锐化".to_string(), "sharpen".to_string(), false),
            ("智能锐化 (人像)".to_string(), "portrait".to_string(), false),
            ("智能降噪".to_string(), "denoise".to_string(), false),
            ("智能降噪 (清洁)".to_string(), "clean".to_string(), false),
            ("胶片颗粒".to_string(), "grain".to_string(), false),
            ("智能柔焦".to_string(), "blur".to_string(), false),
            ("随机色温".to_string(), "color".to_string(), false),
            ("电影暗角".to_string(), "vignette".to_string(), false),
            ("黑白怀旧".to_string(), "bw".to_string(), false),
            ("智能补边".to_string(), "border".to_string(), false),
            ("智能抽帧".to_string(), "pull".to_string(), false),
            ("边角模糊".to_string(), "corner".to_string(), false),
        ]);
        
        // AI与AB模式
        checkboxes.extend(vec![
            ("AI随机缩放".to_string(), "zoom".to_string(), false),
            ("AI移动溶解".to_string(), "dissolve".to_string(), false),
            ("AI随机光扫".to_string(), "scan".to_string(), false),
            ("弹跳效果".to_string(), "bounce".to_string(), false),
            ("三联屏效果".to_string(), "trifold".to_string(), false),
            ("岩浆AB模式".to_string(), "lava".to_string(), false),
            ("3D闪白".to_string(), "flash".to_string(), false),
            ("渐进处理".to_string(), "progressive".to_string(), false),
            ("AB混合模式".to_string(), "ab_blend".to_string(), false),
            ("AB故障效果".to_string(), "ab_glitch".to_string(), false),
            ("AB抖动效果".to_string(), "ab_shake".to_string(), false),
            ("AB色度偏移".to_string(), "ab_chroma".to_string(), false),
            ("AB视频替换".to_string(), "ab_replace".to_string(), false),
            ("高级AB替换".to_string(), "ab_advanced_replace".to_string(), false),
        ]);
        
        // 音频与其他
        checkboxes.extend(vec![
            ("静音视频".to_string(), "mute".to_string(), false),
            ("混入弱白噪音".to_string(), "audio_noise".to_string(), false),
            ("音频变调".to_string(), "pitch".to_string(), false),
            ("仅修改时间戳".to_string(), "touch".to_string(), false),
        ]);
        
        // === 附加功能 (Tab::Additional) ===
        // 强力去重
        checkboxes.extend(vec![
            ("强力裁剪 (8-12%)".to_string(), "strong_crop".to_string(), false),
            ("添加水印".to_string(), "watermark".to_string(), false),
            ("修改编码参数".to_string(), "encode".to_string(), false),
            ("添加贴纸".to_string(), "sticker".to_string(), false),
            ("蒙版叠加".to_string(), "mask".to_string(), false),
            ("蒙版视频叠加".to_string(), "mask_video".to_string(), false),
            ("真实AB替换".to_string(), "ab_real_replace".to_string(), false),
        ]);
        
        // OpenCV功能
        checkboxes.extend(vec![
            ("人脸检测".to_string(), "face_detection".to_string(), false),
            ("物体追踪".to_string(), "object_tracking".to_string(), false),
            ("OpenCV滤镜".to_string(), "opencv_filter".to_string(), false),
        ]);
        
        // 新素材功能
        checkboxes.extend(vec![
            ("光效叠加".to_string(), "light_effect".to_string(), false),
            ("画中画".to_string(), "pip".to_string(), false),
            ("边缘效果".to_string(), "edge_effect".to_string(), false),
            ("带货模板".to_string(), "goods_template".to_string(), false),
        ]);
        
        Self {
            input_dir: String::new(),
            output_dir: String::new(),
            selected_actions: Vec::new(),
            is_processing: false,
            progress: 0.0,
            log_messages: vec![
                "✨ 视频矩阵 Pro 已就绪".to_string(),
                "💡 提示：选择输入目录，勾选功能，然后点击\"开始处理\"".to_string(),
            ],
            
            rx: None,
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
            current_tab: Tab::All,
            
            checkboxes,
            watermark_path: String::new(),
            mask_path: String::new(),
            sticker_path: String::new(),
            border_path: String::new(),
            light_effect_path: String::new(),
            pip_path: String::new(),
            goods_path: String::new(),
            mask_video_path: String::new(),
            action_params: std::collections::HashMap::new(),
            show_settings_dialog: false,
            settings_action_id: String::new(),
            crop_min: 0.01,
            crop_max: 0.05,
            watermark_position: "top_right".to_string(),
            watermark_opacity: 0.5,
            
            // Defaults
            rotate_angle: 1.5,
            speed_range: 0.05, // Conservative: 5% speed variation
            target_fps: 60,
            target_bitrate: "15M".to_string(),
            sharpen_strength: 1.0,
            denoise_strength: 5.0,
            blur_strength: 0.5, // Conservative: very slight blur
            grain_strength: 0.1,
            vignette_strength: 0.2, // Conservative: subtle vignette
            border_width: 20,
            
            // Additional defaults
            cut_seconds: 1.0,
            mirror_direction: "horizontal".to_string(),
            strong_crop_ratio: 0.1,
            portrait_strength: 2.0,
            color_temp_range: 500,
            pull_width: 50,
            progressive_ratio: 0.1,
            corner_radius: 50.0,
            zoom_range: 0.1,
            dissolve_strength: 0.5,
            scan_strength: 0.5,
            bounce_amplitude: 20.0,
            trifold_spacing: 10,
            flash_strength: 0.3,
            lava_strength: 0.5,
            noise_strength: 0.01,
            pitch_range: 0.5, // Conservative: 0.5 semitones
            
            // AI defaults
            deepseek_api_key: String::new(),
            deepseek_base_url: "https://api.deepseek.com".to_string(),
            ai_prompt: String::new(),
            
            // Mask video defaults
            mask_video_opacity: 0.8,
            mask_video_blend_mode: "multiply".to_string(),
            mask_video_scale: "stretch".to_string(),
            
            // 单个视频模式默认关闭
            single_video_mode: false,

            // UI Defaults
            show_ui_settings: false,
            ui_font_scale: 2.0,
            ui_bg_color: [50, 50, 50],
            ui_bg_alpha: 255,
        }
    }
}

impl eframe::App for VideoMatrixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // === Custom Visuals for Better Aesthetics ===
        let mut visuals = egui::Visuals::dark();
        
        // Apply Global UI Settings
        ctx.set_pixels_per_point(self.ui_font_scale);
        
        let bg_color = egui::Color32::from_rgba_premultiplied(
            self.ui_bg_color[0], 
            self.ui_bg_color[1], 
            self.ui_bg_color[2], 
            self.ui_bg_alpha
        );
        
        visuals.window_fill = bg_color;
        visuals.panel_fill = bg_color;
        visuals.widgets.noninteractive.bg_fill = bg_color;
        
        // High contrast text
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        
        visuals.selection.bg_fill = egui::Color32::from_rgb(100, 100, 100); // Grey selection
        ctx.set_visuals(visuals);

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
                        self.log_internal("🎉 所有任务已完成！".to_string());
                        self.progress = 1.0;
                    },
                    AppMessage::Error(e) => {
                        self.log_internal(format!("❌ 错误: {}", e));
                        self.is_processing = false;
                        keep_rx = false;
                    },
                    AppMessage::AIConnectionResult(msg) => {
                        self.log_internal(msg);
                        self.is_processing = false;
                        keep_rx = false;
                    },
                    AppMessage::AIResult(response) => {
                        self.log_internal("✅ AI 分析完成！正在应用推荐设置...".to_string());
                        self.log_internal(format!("💡 AI 建议: {}", response.explanation));
                        
                        // Apply parameters
                        if let Some(obj) = response.params.as_object() {
                            for (k, v) in obj {
                                if let Some(f) = v.as_f64() {
                                    match k.as_str() {
                                        "cut_seconds" => self.cut_seconds = f as f32,
                                        "rotate_angle" => self.rotate_angle = f as f32,
                                        "speed_range" => self.speed_range = f as f32,
                                        "sharpen_strength" => self.sharpen_strength = f as f32,
                                        "denoise_strength" => self.denoise_strength = f as f32,
                                        "blur_strength" => self.blur_strength = f as f32,
                                        "grain_strength" => self.grain_strength = f as f32,
                                        "vignette_strength" => self.vignette_strength = f as f32,
                                        "portrait_strength" => self.portrait_strength = f as f32,
                                        "progressive_ratio" => self.progressive_ratio = f as f32,
                                        "corner_radius" => self.corner_radius = f as f32,
                                        "zoom_range" => self.zoom_range = f as f32,
                                        "dissolve_strength" => self.dissolve_strength = f as f32,
                                        "scan_strength" => self.scan_strength = f as f32,
                                        "bounce_amplitude" => self.bounce_amplitude = f as f32,
                                        "flash_strength" => self.flash_strength = f as f32,
                                        "lava_strength" => self.lava_strength = f as f32,
                                        "noise_strength" => self.noise_strength = f as f32,
                                        "pitch_range" => self.pitch_range = f as f32,
                                        "strong_crop_ratio" => self.strong_crop_ratio = f as f32,
                                        _ => {}
                                    }
                                }
                                if let Some(i) = v.as_i64() {
                                    match k.as_str() {
                                        "border_width" => self.border_width = i as i32,
                                        "color_temp_range" => self.color_temp_range = i as i32,
                                        "pull_width" => self.pull_width = i as i32,
                                        "trifold_spacing" => self.trifold_spacing = i as i32,
                                        "target_fps" => self.target_fps = i as u32,
                                        _ => {}
                                    }
                                }
                                if let Some(s) = v.as_str() {
                                    match k.as_str() {
                                        "target_bitrate" => self.target_bitrate = s.to_string(),
                                        "mirror_direction" => self.mirror_direction = s.to_string(),
                                        _ => {}
                                    }
                                }
                            }
                        }
                        
                        // Select actions
                        self.selected_actions.clear();
                        for action_id in response.suggested_actions {
                            self.selected_actions.push(action_id);
                        }
                        
                        self.log_internal("✨ 设置已更新，您可以点击'开始处理'了！".to_string());
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
            .min_height(180.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    
                    // Control Area
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("已选择 {} 个功能", self.selected_actions.len())).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new("🛑 停止").fill(egui::Color32::from_rgb(200, 50, 50))).clicked() {
                                self.stop_processing();
                            }
                            
                            let can_start = !self.input_dir.is_empty() && !self.selected_actions.is_empty() && !self.is_processing;
                            let start_btn = egui::Button::new("🚀 开始处理").min_size(egui::vec2(120.0, 30.0));
                            
                            // Status Text
                            if !can_start {
                                if self.input_dir.is_empty() {
                                    ui.colored_label(egui::Color32::RED, "⚠️ 请选择输入目录");
                                } else if self.selected_actions.is_empty() {
                                    ui.colored_label(egui::Color32::RED, "⚠️ 请选择功能");
                                } else if self.is_processing {
                                    ui.colored_label(egui::Color32::YELLOW, "⏳ 处理中...");
                                }
                            }

                            if can_start {
                                if ui.add(start_btn.fill(egui::Color32::from_rgb(0, 122, 204))).clicked() {
                                    self.start_processing();
                                }
                            } else {
                                let response = ui.add_enabled(false, start_btn);
                                if self.input_dir.is_empty() {
                                    response.on_disabled_hover_text("请先选择输入目录");
                                } else if self.selected_actions.is_empty() {
                                    response.on_disabled_hover_text("请至少选择一个功能");
                                } else if self.is_processing {
                                    response.on_disabled_hover_text("正在处理中，请稍候");
                                }
                            }
                        });
                    });
                    
                    ui.add_space(8.0);
                    
                    // Progress Bar
                    let progress_bar = egui::ProgressBar::new(self.progress)
                        .show_percentage()
                        .animate(self.is_processing);
                    ui.add(progress_bar);
                    
                    ui.add_space(8.0);
                    ui.separator();
                    
                    // Log Area
                    ui.collapsing("📋 处理日志", |ui| {
                        let text_style = egui::TextStyle::Monospace;
                        let row_height = ui.text_style_height(&text_style);
                        let total_rows = self.log_messages.len();
                        
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .stick_to_bottom(true)
                            .show_rows(ui, row_height, total_rows, |ui, row_range| {
                                for row in row_range {
                                    let msg = &self.log_messages[row];
                                    let color = if msg.contains("Error") || msg.contains("Failed") {
                                        egui::Color32::LIGHT_RED
                                    } else if msg.contains("Completed") || msg.contains("Success") {
                                        egui::Color32::LIGHT_GREEN
                                    } else {
                                        egui::Color32::LIGHT_GRAY
                                    };
                                    ui.colored_label(color, msg);
                                }
                            });
                    });
                    ui.add_space(5.0);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header with Settings Button
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("视频矩阵 Pro").size(24.0).strong());
                ui.label(egui::RichText::new("作者: zwm").size(16.0).color(egui::Color32::LIGHT_BLUE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙️ UI设置").clicked() {
                        self.show_ui_settings = true;
                    }
                    ui.label(egui::RichText::new("v5.5.13").size(14.0).color(egui::Color32::GRAY));
                });
            });
            ui.add_space(10.0);
            
            // UI Settings Dialog
            if self.show_ui_settings {
                egui::Window::new("🎨 界面个性化设置")
                    .collapsible(false)
                    .resizable(false)
                    .pivot(egui::Align2::RIGHT_TOP)
                    .show(ctx, |ui| {
                        ui.heading("界面调整");
                        ui.add_space(8.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("字体大小:");
                            ui.add(egui::Slider::new(&mut self.ui_font_scale, 0.5..=3.0).text("倍率"));
                        });
                        
                        ui.add_space(8.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("背景透明度:");
                            ui.add(egui::Slider::new(&mut self.ui_bg_alpha, 50..=255).text("Alpha"));
                        });
                        
                        ui.add_space(8.0);
                        
                        ui.collapsing("背景颜色", |ui| {
                            ui.color_edit_button_srgb(&mut self.ui_bg_color);
                        });
                        
                        ui.add_space(15.0);
                        if ui.button("关闭").clicked() {
                            self.show_ui_settings = false;
                        }
                    });
            }
            
            // Workspace Section
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(35, 35, 35))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("📁 工作目录");
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("输入:");
                        let _input_response = ui.add(
                            egui::TextEdit::singleline(&mut self.input_dir)
                                .hint_text("选择视频源文件夹...")
                                .desired_width(400.0)
                        );
                        if ui.button("📂 浏览").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.input_dir = path.to_string_lossy().to_string();
                                self.log(&format!("已选择输入目录: {}", self.input_dir));
                            }
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("输出:");
                        let _output_response = ui.add(
                            egui::TextEdit::singleline(&mut self.output_dir)
                                .hint_text("默认：输入目录/output")
                                .desired_width(400.0)
                        );
                        if ui.button("💾 保存到").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.output_dir = path.to_string_lossy().to_string();
                                self.log(&format!("已选择输出目录: {}", self.output_dir));
                            }
                        }
                    });
                });
            
            ui.add_space(15.0);
            
            // Tab Selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::All, "🛠️ 全部功能");
                ui.selectable_value(&mut self.current_tab, Tab::Additional, "✨ 附加功能");
                ui.selectable_value(&mut self.current_tab, Tab::Materials, "🎨 素材设置");
                ui.selectable_value(&mut self.current_tab, Tab::AIDedup, "🤖 AI消重");
                ui.selectable_value(&mut self.current_tab, Tab::ProcessingMode, "🎯 处理模式");
                ui.selectable_value(&mut self.current_tab, Tab::Presets, "💾 配置预设");
                ui.selectable_value(&mut self.current_tab, Tab::Preview, "🎬 效果预览");
                ui.selectable_value(&mut self.current_tab, Tab::Help, "📖 使用说明");
            });
            
            ui.separator();
            
            // Scrollable Area for Features
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                // Collect updates
                let mut updates = Vec::new();
                
                // Show features based on current tab
                match self.current_tab {
                    Tab::All => {
                        self.render_checkbox_group(ui, "✂️ 基础编辑", 0..8, &mut updates);
                        ui.add_space(10.0);
                        self.render_checkbox_group(ui, "🎨 视觉增强", 8..20, &mut updates);
                        ui.add_space(10.0);
                        self.render_checkbox_group(ui, "🤖 AI与AB模式", 20..34, &mut updates);
                        ui.add_space(10.0);
                        self.render_checkbox_group(ui, "🎵 音频与其他", 34..38, &mut updates);
                    }
                    Tab::Additional => {
                        self.render_checkbox_group(ui, "💪 强力去重", 38..44, &mut updates);
                        ui.add_space(10.0);
                        self.render_checkbox_group(ui, "👁️ OpenCV功能", 44..47, &mut updates);
                        ui.add_space(10.0);
                        self.render_checkbox_group(ui, "✨ 新素材功能", 47..51, &mut updates);
                    }
                    Tab::Materials => {
                        ui.heading("🎨 素材设置");
                        ui.add_space(10.0);
                        
                        // 水印素材
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.label("水印图片:");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.watermark_path).hint_text("选择图片...").desired_width(400.0));
                                if ui.button("浏览").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("图片", &["png", "jpg", "jpeg"]).pick_file() {
                                        self.watermark_path = path.to_string_lossy().to_string();
                                        self.log(&format!("已选择水印: {}", self.watermark_path));
                                    }
                                }
                            });
                            ui.small("支持格式：PNG (推荐), JPG");
                        });
                        
                        ui.add_space(10.0);
                        
                        // 蒙版素材
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.label("蒙版图片:");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.mask_path).hint_text("选择图片...").desired_width(400.0));
                                if ui.button("浏览").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("图片", &["png", "jpg"]).pick_file() {
                                        self.mask_path = path.to_string_lossy().to_string();
                                        self.log(&format!("已选择蒙版: {}", self.mask_path));
                                    }
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // 蒙版视频素材
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.label("蒙版视频:");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.mask_video_path).hint_text("选择视频...").desired_width(400.0));
                                if ui.button("浏览").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("视频", &["mp4", "mov", "avi"]).pick_file() {
                                        self.mask_video_path = path.to_string_lossy().to_string();
                                        self.log(&format!("已选择蒙版视频: {}", self.mask_video_path));
                                    }
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // 贴纸素材
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.label("贴纸图片:");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.sticker_path).hint_text("选择图片...").desired_width(400.0));
                                if ui.button("浏览").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("图片", &["png", "gif"]).pick_file() {
                                        self.sticker_path = path.to_string_lossy().to_string();
                                        self.log(&format!("已选择贴纸: {}", self.sticker_path));
                                    }
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // 边框素材
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.label("边框图片:");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.border_path).hint_text("选择图片...").desired_width(400.0));
                                if ui.button("浏览").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("图片", &["png"]).pick_file() {
                                        self.border_path = path.to_string_lossy().to_string();
                                        self.log(&format!("已选择边框: {}", self.border_path));
                                    }
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // 光效素材
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.label("光效素材:");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.light_effect_path).hint_text("选择视频或图片...").desired_width(400.0));
                                if ui.button("浏览").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("媒体", &["mp4", "mov", "png"]).pick_file() {
                                        self.light_effect_path = path.to_string_lossy().to_string();
                                        self.log(&format!("已选择光效: {}", self.light_effect_path));
                                    }
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // 画中画素材
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.label("画中画视频:");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.pip_path).hint_text("选择视频...").desired_width(400.0));
                                if ui.button("浏览").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("视频", &["mp4", "mov", "avi"]).pick_file() {
                                        self.pip_path = path.to_string_lossy().to_string();
                                        self.log(&format!("已选择画中画: {}", self.pip_path));
                                    }
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // 带货模板素材
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.label("带货模板:");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.goods_path).hint_text("选择模板...").desired_width(400.0));
                                if ui.button("浏览").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().add_filter("媒体", &["mp4", "png"]).pick_file() {
                                        self.goods_path = path.to_string_lossy().to_string();
                                        self.log(&format!("已选择模板: {}", self.goods_path));
                                    }
                                }
                            });
                        });
                    }
                    Tab::Presets => self.render_presets_tab(ui),
                    Tab::Preview => self.render_preview_tab(ui),
                    
                    Tab::Help => {
                        ui.heading("📖 使用说明");
                        ui.add_space(10.0);
                        
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.label("欢迎使用视频矩阵 Pro！本工具提供 51 种视频处理功能，帮助您快速批量处理视频。");
                                ui.add_space(10.0);
                                
                                // 环境要求
                                egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                                    ui.heading("⚠️ 环境要求");
                                    ui.add_space(5.0);
                                    ui.label("本软件依赖 FFmpeg 进行视频处理，请确保：");
                                    ui.label("1. 已安装 FFmpeg");
                                    ui.label("2. FFmpeg 已添加到系统环境变量 PATH 中");
                                    ui.label("3. 在终端输入 'ffmpeg -version' 能正常显示版本信息");
                                    ui.add_space(5.0);
                                    ui.hyperlink("https://ffmpeg.org/download.html");
                                });
                                ui.add_space(15.0);
                                
                                // 基础使用
                                ui.heading("🚀 快速开始");
                                ui.label("1. 确保已安装 FFmpeg（见上文）");
                                ui.label("2. 选择输入目录（包含要处理的视频文件）");
                                ui.label("3. 勾选需要的功能（可多选）");
                                ui.label("4. 点击功能旁的 ⚙️ 按钮调整参数（可选）");
                                ui.label("5. 点击\"开始处理\"按钮");
                                ui.label("6. 处理完成后，视频将保存在输出目录");
                                ui.add_space(15.0);
                                
                                // 功能分类说明
                                ui.heading("📚 功能详解");
                                ui.add_space(5.0);
                                
                                // 基础编辑
                                egui::CollapsingHeader::new("🔧 基础编辑 (8个)")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("• 一键MD5：修改视频元数据，添加唯一标识");
                                        ui.label("• 随机微裁剪：随机裁剪视频边缘（可调节比例）");
                                        ui.label("• 首尾去秒：去除视频开头和结尾的指定秒数");
                                        ui.label("• 微旋转：随机旋转视频（可调节角度范围）");
                                        ui.label("• 非线性变速：随机调整播放速度（可调节范围）");
                                        ui.label("• 镜像翻转：水平/垂直/双向翻转视频");
                                        ui.label("• 强制60帧：将视频转换为指定帧率");
                                        ui.label("• 高码率：提升视频码率，增强画质");
                                    });
                                
                                ui.add_space(5.0);
                                
                                // 视觉增强
                                egui::CollapsingHeader::new("✨ 视觉增强 (12个)")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("• 智能锐化：增强视频清晰度（可调节强度）");
                                        ui.label("• 智能锐化(人像)：针对人像优化的锐化");
                                        ui.label("• 智能降噪：减少视频噪点（可调节强度）");
                                        ui.label("• 智能降噪(清洁)：更强的降噪效果");
                                        ui.label("• 胶片颗粒：添加电影感颗粒效果（可调节强度）");
                                        ui.label("• 智能柔焦：柔化画面，营造梦幻效果");
                                        ui.label("• 随机色温：调整视频色温");
                                        ui.label("• 电影暗角：添加四周暗角效果（可调节强度）");
                                        ui.label("• 黑白怀旧：转换为黑白效果");
                                        ui.label("• 智能补边：为视频添加边框");
                                        ui.label("• 智能抽帧：降低帧率，减小文件大小");
                                        ui.label("• 边角模糊：模糊视频四角");
                                    });
                                
                                ui.add_space(5.0);
                                
                                // 强力去重
                                egui::CollapsingHeader::new("🔥 强力去重 (6个)")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("• 强力裁剪：大幅度裁剪，强力去重");
                                        ui.label("• 添加水印：叠加水印图片（可调位置和透明度）");
                                        ui.label("• 修改编码参数：更改视频编码设置");
                                        ui.label("• 添加贴纸：叠加贴纸素材");
                                        ui.label("• 蒙版叠加：应用蒙版效果");
                                        ui.label("• 真实AB替换：高级AB模式替换");
                                    });
                                
                                ui.add_space(5.0);
                                
                                // AI与AB模式
                                egui::CollapsingHeader::new("🤖 AI与AB模式 (14个)")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("• AI随机缩放：智能缩放视频");
                                        ui.label("• AI移动溶解：动态溶解效果");
                                        ui.label("• AI随机光扫：光线扫描效果");
                                        ui.label("• 弹跳效果：视频弹跳动画");
                                        ui.label("• 三联屏效果：分屏显示");
                                        ui.label("• 岩浆AB模式：岩浆风格特效");
                                        ui.label("• 3D闪白：3D闪光效果");
                                        ui.label("• 渐进处理：渐进式视频处理");
                                        ui.label("• AB混合模式：混合两个视频");
                                        ui.label("• AB故障效果：故障艺术风格");
                                        ui.label("• AB抖动效果：抖动特效");
                                        ui.label("• AB色度偏移：色彩偏移效果");
                                        ui.label("• AB视频替换：替换视频片段");
                                        ui.label("• 高级AB替换：更高级的替换模式");
                                    });
                                
                                ui.add_space(5.0);
                                
                                // 素材叠加
                                egui::CollapsingHeader::new("🎨 素材叠加 (7个)")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("• 水印：在\"素材设置\"中选择水印图片");
                                        ui.label("• 贴纸：在\"素材设置\"中选择贴纸图片");
                                        ui.label("• 蒙版：在\"素材设置\"中选择蒙版图片");
                                        ui.label("• 边框：在\"素材设置\"中选择边框图片");
                                        ui.label("• 光效：在\"素材设置\"中选择光效素材");
                                        ui.label("• 画中画：在\"素材设置\"中选择叠加视频");
                                        ui.label("• 带货模板：在\"素材设置\"中选择模板");
                                    });
                                
                                ui.add_space(5.0);
                                
                                // 音频处理
                                egui::CollapsingHeader::new("🎵 音频处理 (4个)")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("• 静音视频：移除所有音频");
                                        ui.label("• 混入弱白噪音：添加背景白噪音（可调强度）");
                                        ui.label("• 音频变调：随机调整音调（可调范围）");
                                        ui.label("• 仅修改时间戳：只更改元数据时间戳");
                                    });
                                
                                ui.add_space(5.0);
                                
                                // AI 智能消重
                                egui::CollapsingHeader::new("🤖 AI 智能消重")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.label("• 智能分析：AI 自动分析视频处理需求");
                                        ui.label("• 自动推荐：根据需求推荐最佳功能组合");
                                        ui.label("• 参数优化：自动设置最合适的处理参数");
                                        ui.label("• 使用方法：切换到'AI消重'标签页，输入 Key 和需求即可");
                                    });
                                
                                ui.add_space(15.0);
                                
                                // 使用技巧
                                ui.heading("💡 使用技巧");
                                ui.label("• 可以同时勾选多个功能，按顺序依次处理");
                                ui.label("• 点击 ⚙️ 按钮可精细调节每个功能的参数");
                                ui.label("• 建议先用少量视频测试效果，再批量处理");
                                ui.label("• 处理过程中可查看\"日志\"标签页了解进度");
                                ui.label("• 素材功能需要先在\"素材设置\"中选择对应文件");
                                
                                ui.add_space(15.0);
                                
                                // 注意事项
                                ui.heading("⚠️ 注意事项");
                                ui.label("• 确保有足够的磁盘空间存储输出文件");
                                ui.label("• 处理大量视频时可能需要较长时间");
                                ui.label("• 某些功能组合可能导致处理时间增加");
                                ui.label("• 建议定期备份原始视频文件");
                            });
                    }
                    
                    Tab::AIDedup => {
                        ui.heading("🤖 AI 智能消重");
                        ui.add_space(10.0);
                        
                        ui.label("使用 AI 大模型智能分析视频内容，生成个性化的处理方案");
                        ui.add_space(15.0);
                        
                        // API 配置区域
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.heading("🔑 API 配置");
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("DeepSeek API Key:");
                                ui.add(egui::TextEdit::singleline(&mut self.deepseek_api_key)
                                    .hint_text("sk-xxxxxxxxxxxxxxxx")
                                    .password(true)
                                    .desired_width(400.0));
                            });
                            ui.small("在 https://platform.deepseek.com 获取 API Key");
                            
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("API Base URL:");
                                ui.add(egui::TextEdit::singleline(&mut self.deepseek_base_url)
                                    .hint_text("https://api.deepseek.com")
                                    .desired_width(400.0));
                            });
                            ui.small("通常使用默认值即可");
                        });
                        
                        ui.add_space(15.0);
                        
                        // AI 提示词区域
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.heading("💬 AI 处理需求");
                            ui.add_space(5.0);
                            
                            ui.label("描述您希望 AI 如何处理视频（例如：去重、风格化、特效等）");
                            ui.add_space(5.0);
                            
                            ui.add(egui::TextEdit::multiline(&mut self.ai_prompt)
                                .hint_text("例如：\n- 分析视频内容，自动添加合适的滤镜和特效\n- 识别重复片段并进行智能剪辑\n- 根据视频主题推荐最佳的处理参数\n- 生成创意转场效果")
                                .desired_width(f32::INFINITY)
                                .desired_rows(8));
                        });
                        
                        ui.add_space(15.0);
                        
                        // 功能说明
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.heading("📚 功能说明");
                            ui.add_space(5.0);
                            
                            ui.label("• AI 会分析您的需求和视频内容");
                            ui.label("• 自动选择合适的处理功能和参数");
                            ui.label("• 生成个性化的视频处理方案");
                            ui.label("• 支持批量处理和智能优化");
                            
                            ui.add_space(10.0);
                            
                            ui.label("⚠️ 注意：");
                            ui.label("• 需要有效的 DeepSeek API Key");
                            ui.label("• API 调用可能产生费用");
                            ui.label("• 处理时间取决于视频数量和复杂度");
                        });
                        
                        ui.add_space(15.0);
                        
                        // 操作按钮
                        ui.horizontal(|ui| {
                            if ui.button("🚀 开始 AI 处理").clicked() {
                                if self.deepseek_api_key.is_empty() {
                                    self.log("❌ 请先配置 DeepSeek API Key");
                                } else if self.ai_prompt.is_empty() {
                                    self.log("❌ 请输入 AI 处理需求");
                                } else {
                                    self.log("🤖 正在请求 AI 分析...");
                                    self.is_processing = true;
                                    
                                    let api_key = self.deepseek_api_key.clone();
                                    let base_url = self.deepseek_base_url.clone();
                                    let prompt = self.ai_prompt.clone();
                                    let (tx, rx) = channel();
                                    self.rx = Some(rx);
                                    let tx = tx.clone();
                                    
                                    self.runtime.spawn(async move {
                                        let service = AIService::new(api_key, base_url);
                                        match service.analyze_requirement(&prompt).await {
                                            Ok(response) => {
                                                let _ = tx.send(AppMessage::AIResult(response));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(AppMessage::Error(format!("AI 请求失败: {}", e)));
                                            }
                                        }
                                    });
                                }
                            }
                            
                            if ui.button("🧪 测试连接").clicked() {
                                if self.deepseek_api_key.is_empty() {
                                    self.log("❌ 请先配置 API Key");
                                } else {
                                    self.log("🔍 正在测试 API 连接...");
                                    self.is_processing = true;
                                    
                                    let api_key = self.deepseek_api_key.clone();
                                    let base_url = self.deepseek_base_url.clone();
                                    let (tx, rx) = channel();
                                    self.rx = Some(rx);
                                    let tx = tx.clone();
                                    
                                    self.runtime.spawn(async move {
                                        let service = AIService::new(api_key, base_url);
                                        match service.test_connection().await {
                                            Ok(msg) => {
                                                let _ = tx.send(AppMessage::AIConnectionResult(msg));
                                            }
                                            Err(e) => {
                                                let _ = tx.send(AppMessage::Error(format!("连接失败: {}", e)));
                                            }
                                        }
                                    });
                                }
                            }
                            
                            if ui.button("🔄 重置配置").clicked() {
                                self.deepseek_api_key.clear();
                                self.deepseek_base_url = "https://api.deepseek.com".to_string();
                                self.ai_prompt.clear();
                                self.log("✅ 已重置 AI 配置");
                            }
                        });
                    }
                    
                    Tab::ProcessingMode => {
                        ui.heading("🎯 处理模式设置");
                        ui.add_space(10.0);
                        
                        ui.label("设置视频处理的工作模式，此设置对所有标签页的功能都有效");
                        ui.add_space(15.0);
                        
                        // 单个视频功能叠加模式开关
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.heading("📽️ 视频输出模式");
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut self.single_video_mode, "单个视频功能叠加模式").changed() {
                                    self.log(&format!("{} 单个视频功能叠加模式", 
                                        if self.single_video_mode { "✅ 已开启" } else { "✅ 已关闭" }));
                                }
                            });
                            
                            ui.add_space(5.0);
                            ui.label("• 开启：所有选中的功能按顺序应用到同一个视频，最终生成一个文件");
                            ui.label("• 关闭：每个功能生成独立的视频文件（原始模式）");
                            ui.small("⚠️ 注意：请在开始处理前设置此选项");
                        });
                        
                        ui.add_space(15.0);
                        
                        // 模式说明
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.heading("📚 模式说明");
                            ui.add_space(5.0);
                            
                            ui.label("🔹 单个视频功能叠加模式（推荐用于去重）");
                            ui.label("   • 所有选中的功能按顺序应用到同一个视频");
                            ui.label("   • 最终只生成一个处理后的视频文件");
                            ui.label("   • 适合需要多重处理的场景");
                            ui.label("   • 文件命名：原文件名_processed.扩展名");
                            
                            ui.add_space(10.0);
                            
                            ui.label("🔹 独立视频输出模式（原始模式）");
                            ui.label("   • 每个功能生成独立的视频文件");
                            ui.label("   • 适合需要单独查看每个效果的情况");
                            ui.label("   • 文件命名：原文件名_功能名.扩展名");
                            
                            ui.add_space(5.0);
                            ui.small("💡 提示：切换模式后，已选中的功能不会改变");
                        });
                        
                        ui.add_space(15.0);
                        
                        // 当前状态显示
                        egui::Frame::group(ui.style()).inner_margin(10.0).show(ui, |ui| {
                            ui.heading("📊 当前状态");
                            ui.add_space(5.0);
                            
                            ui.label(format!("当前模式：{}", 
                                if self.single_video_mode { 
                                    "✅ 单个视频功能叠加模式" 
                                } else { 
                                    "✅ 独立视频输出模式" 
                                }));
                            
                            ui.label(format!("已选中功能：{} 个", self.selected_actions.len()));
                            if !self.selected_actions.is_empty() {
                                ui.label("选中的功能：");
                                for action in &self.selected_actions {
                                    ui.label(format!("  • {}", action));
                                }
                            }
                        });
                    }
                }
                
                
                // Process updates
                for (id, name, _old_checked, new_checked) in updates {
                    if new_checked {
                        self.selected_actions.push(id);
                    } else {
                        self.selected_actions.retain(|x| x != &id);
                    }
                    self.log(&format!("{} {}", if new_checked { "已选择" } else { "已取消" }, name));
                }
            });
        });
        
        // Settings Dialog
        if self.show_settings_dialog {
            egui::Window::new("参数设置")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    match self.settings_action_id.as_str() {
                        "crop" => {
                            ui.heading("随机微裁剪设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("最小比例:");
                                ui.add(egui::DragValue::new(&mut self.crop_min).speed(0.001).clamp_range(0.0..=0.5));
                            });
                            ui.horizontal(|ui| {
                                ui.label("最大比例:");
                                ui.add(egui::DragValue::new(&mut self.crop_max).speed(0.001).clamp_range(0.0..=0.5));
                            });
                            ui.small("范围: 0.0 - 0.5 (例如 0.05 代表 5%)");
                        },
                        "rotate" => {
                            ui.heading("微旋转设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("最大角度:");
                                ui.add(egui::Slider::new(&mut self.rotate_angle, 0.1..=10.0).text("度"));
                            });
                            ui.small("视频将在此范围内随机旋转");
                        },
                        "speed" => {
                            ui.heading("变速设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("变速范围:");
                                ui.add(egui::Slider::new(&mut self.speed_range, 0.01..=0.5).text("幅度"));
                            });
                            ui.small("例如 0.1 代表速度在 0.9x 到 1.1x 之间随机");
                        },
                        "fps" => {
                            ui.heading("帧率设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("目标帧率:");
                                ui.selectable_value(&mut self.target_fps, 30, "30 FPS");
                                ui.selectable_value(&mut self.target_fps, 60, "60 FPS");
                            });
                        },
                        "bitrate" => {
                            ui.heading("码率设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("目标码率:");
                                ui.text_edit_singleline(&mut self.target_bitrate);
                            });
                            ui.small("例如: 10M, 15M, 5000k");
                        },
                        "sharpen" => {
                            ui.heading("锐化设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("强度:");
                                ui.add(egui::Slider::new(&mut self.sharpen_strength, 0.0..=5.0));
                            });
                        },
                        "denoise" => {
                            ui.heading("降噪设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("强度:");
                                ui.add(egui::Slider::new(&mut self.denoise_strength, 0.0..=20.0));
                            });
                        },
                        "blur" => {
                            ui.heading("模糊设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("强度 (Sigma):");
                                ui.add(egui::Slider::new(&mut self.blur_strength, 0.1..=10.0));
                            });
                        },
                        "grain" => {
                            ui.heading("颗粒设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("强度:");
                                ui.add(egui::Slider::new(&mut self.grain_strength, 0.0..=0.5));
                            });
                        },
                        "vignette" => {
                            ui.heading("暗角设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("强度:");
                                ui.add(egui::Slider::new(&mut self.vignette_strength, 0.1..=1.0));
                            });
                        },
                        "border" => {
                            ui.heading("边框设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("宽度 (像素):");
                                ui.add(egui::DragValue::new(&mut self.border_width).speed(1).clamp_range(0..=500));
                            });
                            ui.small("仅在使用默认模糊边框时有效");
                        },
                        "watermark" => {
                            ui.heading("水印设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("位置:");
                                egui::ComboBox::from_id_source("wm_pos")
                                    .selected_text(match self.watermark_position.as_str() {
                                        "top_left" => "左上",
                                        "top_right" => "右上",
                                        "bottom_left" => "左下",
                                        "bottom_right" => "右下",
                                        "center" => "居中",
                                        _ => "右上"
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.watermark_position, "top_left".to_string(), "左上");
                                        ui.selectable_value(&mut self.watermark_position, "top_right".to_string(), "右上");
                                        ui.selectable_value(&mut self.watermark_position, "bottom_left".to_string(), "左下");
                                        ui.selectable_value(&mut self.watermark_position, "bottom_right".to_string(), "右下");
                                        ui.selectable_value(&mut self.watermark_position, "center".to_string(), "居中");
                                    });
                            });
                            ui.horizontal(|ui| {
                                ui.label("透明度:");
                                ui.add(egui::Slider::new(&mut self.watermark_opacity, 0.1..=1.0).text("不透明度"));
                            });
                        },
                        "mask_video" => {
                            ui.heading("蒙版视频设置");
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("透明度:");
                                ui.add(egui::Slider::new(&mut self.mask_video_opacity, 0.0..=1.0).text("强度"));
                            });
                            
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_source("mask_blend")
                                    .selected_text(&self.mask_video_blend_mode)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.mask_video_blend_mode, "multiply".to_string(), "正片叠底 (Multiply)");
                                        ui.selectable_value(&mut self.mask_video_blend_mode, "screen".to_string(), "滤色 (Screen)");
                                        ui.selectable_value(&mut self.mask_video_blend_mode, "overlay".to_string(), "叠加 (Overlay)");
                                        ui.selectable_value(&mut self.mask_video_blend_mode, "add".to_string(), "相加 (Add)");
                                        ui.selectable_value(&mut self.mask_video_blend_mode, "subtract".to_string(), "相减 (Subtract)");
                                        ui.selectable_value(&mut self.mask_video_blend_mode, "difference".to_string(), "差值 (Difference)");
                                    });
                            });
                            
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label("缩放模式:");
                                egui::ComboBox::from_id_source("mask_scale")
                                    .selected_text(&self.mask_video_scale)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.mask_video_scale, "stretch".to_string(), "拉伸填充");
                                        ui.selectable_value(&mut self.mask_video_scale, "fit".to_string(), "等比缩放");
                                        ui.selectable_value(&mut self.mask_video_scale, "crop".to_string(), "裁剪填充");
                                    });
                            });
                            
                            ui.add_space(5.0);
                            
                            ui.label("💡 提示:");
                            ui.label("• 正片叠底：适合暗色蒙版");
                            ui.label("• 滤色：适合亮色蒙版");
                            ui.label("• 叠加：平衡的混合效果");
                            ui.label("• 相加：增强亮度");
                        },
                        // Basic editing
                        "cut" => {
                            ui.heading("首尾去秒设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("去除秒数:");
                                ui.add(egui::Slider::new(&mut self.cut_seconds, 0.1..=10.0).text("秒"));
                            });
                            ui.small("从视频开头和结尾各去除指定秒数");
                        },
                        "mirror" => {
                            ui.heading("镜像翻转设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("翻转方向:");
                                ui.selectable_value(&mut self.mirror_direction, "horizontal".to_string(), "水平");
                                ui.selectable_value(&mut self.mirror_direction, "vertical".to_string(), "垂直");
                                ui.selectable_value(&mut self.mirror_direction, "both".to_string(), "双向");
                            });
                        },
                        "strong_crop" => {
                            ui.heading("强力裁剪设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("裁剪比例:");
                                ui.add(egui::Slider::new(&mut self.strong_crop_ratio, 0.05..=0.3));
                            });
                            ui.small("裁剪比例越大，去重效果越强");
                        },
                        // Visual enhancements
                        "portrait" => {
                            ui.heading("智能柔焦设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("柔焦强度:");
                                ui.add(egui::Slider::new(&mut self.portrait_strength, 0.5..=10.0));
                            });
                        },
                        "color" => {
                            ui.heading("随机色温设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("色温范围:");
                                ui.add(egui::Slider::new(&mut self.color_temp_range, 100..=2000).text("K"));
                            });
                            ui.small("色温调整范围（开尔文）");
                        },
                        "pull" => {
                            ui.heading("智能补边设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("补边宽度:");
                                ui.add(egui::Slider::new(&mut self.pull_width, 10..=200).text("像素"));
                            });
                        },
                        "progressive" => {
                            ui.heading("渐进处理设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("抽帧比例:");
                                ui.add(egui::Slider::new(&mut self.progressive_ratio, 0.05..=0.5));
                            });
                            ui.small("比例越大，抽帧越多");
                        },
                        "corner" => {
                            ui.heading("边角模糊设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("模糊半径:");
                                ui.add(egui::Slider::new(&mut self.corner_radius, 10.0..=200.0).text("像素"));
                            });
                        },
                        // AI & Effects
                        "zoom" => {
                            ui.heading("AI随机缩放设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("缩放范围:");
                                ui.add(egui::Slider::new(&mut self.zoom_range, 0.01..=0.3));
                            });
                            ui.small("例如 0.1 代表 0.9x 到 1.1x 之间随机缩放");
                        },
                        "dissolve" => {
                            ui.heading("移动溶解设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("溶解强度:");
                                ui.add(egui::Slider::new(&mut self.dissolve_strength, 0.1..=1.0));
                            });
                        },
                        "scan" => {
                            ui.heading("随机光扫设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("光扫强度:");
                                ui.add(egui::Slider::new(&mut self.scan_strength, 0.1..=1.0));
                            });
                        },
                        "bounce" => {
                            ui.heading("弹跳效果设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("弹跳幅度:");
                                ui.add(egui::Slider::new(&mut self.bounce_amplitude, 5.0..=100.0).text("像素"));
                            });
                        },
                        "trifold" => {
                            ui.heading("三联屏设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("屏幕间距:");
                                ui.add(egui::Slider::new(&mut self.trifold_spacing, 0..=50).text("像素"));
                            });
                        },
                        "flash" => {
                            ui.heading("3D闪白设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("闪白强度:");
                                ui.add(egui::Slider::new(&mut self.flash_strength, 0.1..=1.0));
                            });
                        },
                        "lava" => {
                            ui.heading("岩浆AB模式设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("效果强度:");
                                ui.add(egui::Slider::new(&mut self.lava_strength, 0.1..=1.0));
                            });
                        },
                        // Audio
                        "noise" => {
                            ui.heading("白噪音设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("噪音强度:");
                                ui.add(egui::Slider::new(&mut self.noise_strength, 0.001..=0.1));
                            });
                            ui.small("强度越大，噪音越明显");
                        },
                        "pitch" => {
                            ui.heading("音频变调设置");
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("变调范围:");
                                ui.add(egui::Slider::new(&mut self.pitch_range, 0.5..=12.0).text("半音"));
                            });
                            ui.small("±半音数，例如 2 代表 -2 到 +2 半音");
                        },
                        "md5" | "clean" | "mute" => {
                            ui.label("此功能无需参数设置");
                        },
                        _ => {
                            ui.label("此功能暂无参数设置");
                        }
                    }
                    
                    ui.add_space(10.0);
                    if ui.button("关闭").clicked() {
                        self.show_settings_dialog = false;
                    }
                });
        }
        
        // Request repaint to keep UI responsive during processing
        if self.is_processing {
            ctx.request_repaint();
        }
    }
}

// Separate implementation block for the main UI update to keep the file clean
impl VideoMatrixApp {
    fn render_presets_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("💾 配置预设");
        ui.add_space(10.0);
        ui.label("在此处保存和加载您常用的功能组合。");
        ui.add_space(10.0);
        
        egui::Grid::new("presets_grid").num_columns(2).spacing([20.0, 10.0]).show(ui, |ui| {
            ui.label("强力去重模式");
            if ui.button("加载").clicked() {
                self.selected_actions = vec!["md5".to_string(), "crop".to_string(), "cut_head_tail".to_string(), "rotate".to_string(), "speed".to_string()];
                self.single_video_mode = true; // Presets often imply a combined effect
                self.log_internal("✅ 已加载预设: 强力去重模式 (已切换到全部功能页)".to_string());
                self.current_tab = Tab::All;
            }
            ui.end_row();

            ui.label("复古老电影风");
            if ui.button("加载").clicked() {
                self.selected_actions = vec!["bw".to_string(), "grain".to_string(), "vignette".to_string(), "fps_60".to_string()];
                self.single_video_mode = true;
                self.log_internal("✅ 已加载预设: 复古老电影风 (已切换到全部功能页)".to_string());
                self.current_tab = Tab::All;
            }
            ui.end_row();

            ui.label("带货快节奏");
            if ui.button("加载").clicked() {
                self.selected_actions = vec!["speed".to_string(), "sharpen".to_string(), "color".to_string(), "audio_noise".to_string()];
                self.single_video_mode = true;
                 self.log_internal("✅ 已加载预设: 带货快节奏 (已切换到全部功能页)".to_string());
                 self.current_tab = Tab::All;
            }
            ui.end_row();
        });
        
        ui.add_space(20.0);
        
        // Manual save/load (Simplified)
        ui.separator();
        ui.label("自定义预设:");
        ui.horizontal(|ui| {
             if ui.button("保存当前配置").clicked() {
                 self.log_internal("💾 配置保存功能开发中...".to_string());
             }
        });
    }

    fn render_preview_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎬 效果预览");
        ui.add_space(10.0);
        
        if self.input_dir.is_empty() {
             ui.colored_label(egui::Color32::RED, "⚠️ 请先选择输入目录");
        } else {
             ui.label(format!("当前输入: {}", self.input_dir));
             ui.label(format!("当前输出: {}", if self.output_dir.is_empty() { format!("{}/output", self.input_dir) } else { self.output_dir.clone() }));
             ui.add_space(10.0);
             
             let btn_label = if self.is_processing { "⏳ 生成中..." } else { "▶️ 生成 5秒 预览片段" };
             
             if ui.add_enabled(!self.is_processing, egui::Button::new(btn_label).min_size(egui::vec2(150.0, 40.0))).clicked() {
                 self.start_preview_processing();
             }
             
             ui.add_space(10.0);
             ui.info_message("预览逻辑: \n1. 选取第一个视频文件\n2. 截取前 5 秒\n3. 叠加应用所有勾选的功能\n4. 自动打开播放结果");
        }
    }
    
    fn start_preview_processing(&mut self) {
        if self.selected_actions.is_empty() {
            self.log_internal("⚠️ 请先至少选择一个功能".to_string());
            return;
        }

        self.is_processing = true;
        self.progress = 0.0;
        self.log("🎬 开始生成预览...");
        
        // Clone necessary data for the thread
        let input_dir = self.input_dir.clone();
        let output_dir = if self.output_dir.is_empty() {
            format!("{}/output", self.input_dir)
        } else {
            self.output_dir.clone()
        };
        let selected_actions = self.selected_actions.clone();
        
        // Config creation (similar to start_processing)
        let mut config = ActionConfig::default();
        if !self.watermark_path.is_empty() { config.watermark_path = Some(self.watermark_path.clone()); }
        if !self.mask_path.is_empty() { config.mask_path = Some(self.mask_path.clone()); }
        if !self.sticker_path.is_empty() { config.sticker_path = Some(self.sticker_path.clone()); }
        if !self.border_path.is_empty() { config.border_path = Some(self.border_path.clone()); }
        if !self.light_effect_path.is_empty() { config.light_effect_path = Some(self.light_effect_path.clone()); }
        if !self.pip_path.is_empty() { config.pip_path = Some(self.pip_path.clone()); }
        if !self.goods_path.is_empty() { config.goods_path = Some(self.goods_path.clone()); }
        if !self.mask_video_path.is_empty() { config.mask_video_path = Some(self.mask_video_path.clone()); }
        
        // Copy parameters
        config.params.as_object_mut().unwrap().insert("crop_min".to_string(), serde_json::json!(self.crop_min));
        config.params.as_object_mut().unwrap().insert("crop_max".to_string(), serde_json::json!(self.crop_max));
        config.params.as_object_mut().unwrap().insert("watermark_position".to_string(), serde_json::json!(self.watermark_position));
        config.params.as_object_mut().unwrap().insert("watermark_opacity".to_string(), serde_json::json!(self.watermark_opacity));
        config.params.as_object_mut().unwrap().insert("rotate_angle".to_string(), serde_json::json!(self.rotate_angle));
        config.params.as_object_mut().unwrap().insert("speed_range".to_string(), serde_json::json!(self.speed_range));
        config.params.as_object_mut().unwrap().insert("target_fps".to_string(), serde_json::json!(self.target_fps));
        config.params.as_object_mut().unwrap().insert("target_bitrate".to_string(), serde_json::json!(self.target_bitrate));
        config.params.as_object_mut().unwrap().insert("sharpen_strength".to_string(), serde_json::json!(self.sharpen_strength));
        config.params.as_object_mut().unwrap().insert("denoise_strength".to_string(), serde_json::json!(self.denoise_strength));
        config.params.as_object_mut().unwrap().insert("blur_strength".to_string(), serde_json::json!(self.blur_strength));
        config.params.as_object_mut().unwrap().insert("grain_strength".to_string(), serde_json::json!(self.grain_strength));
        config.params.as_object_mut().unwrap().insert("vignette_strength".to_string(), serde_json::json!(self.vignette_strength));
        config.params.as_object_mut().unwrap().insert("border_width".to_string(), serde_json::json!(self.border_width));
        config.params.as_object_mut().unwrap().insert("cut_seconds".to_string(), serde_json::json!(self.cut_seconds));
        config.params.as_object_mut().unwrap().insert("mirror_direction".to_string(), serde_json::json!(self.mirror_direction));
        config.params.as_object_mut().unwrap().insert("strong_crop_ratio".to_string(), serde_json::json!(self.strong_crop_ratio));
        config.params.as_object_mut().unwrap().insert("portrait_strength".to_string(), serde_json::json!(self.portrait_strength));
        config.params.as_object_mut().unwrap().insert("color_temp_range".to_string(), serde_json::json!(self.color_temp_range));
        config.params.as_object_mut().unwrap().insert("pull_width".to_string(), serde_json::json!(self.pull_width));
        config.params.as_object_mut().unwrap().insert("progressive_ratio".to_string(), serde_json::json!(self.progressive_ratio));
        config.params.as_object_mut().unwrap().insert("corner_radius".to_string(), serde_json::json!(self.corner_radius));
        config.params.as_object_mut().unwrap().insert("zoom_range".to_string(), serde_json::json!(self.zoom_range));
        config.params.as_object_mut().unwrap().insert("dissolve_strength".to_string(), serde_json::json!(self.dissolve_strength));
        config.params.as_object_mut().unwrap().insert("scan_strength".to_string(), serde_json::json!(self.scan_strength));
        config.params.as_object_mut().unwrap().insert("bounce_amplitude".to_string(), serde_json::json!(self.bounce_amplitude));
        config.params.as_object_mut().unwrap().insert("trifold_spacing".to_string(), serde_json::json!(self.trifold_spacing));
        config.params.as_object_mut().unwrap().insert("flash_strength".to_string(), serde_json::json!(self.flash_strength));
        config.params.as_object_mut().unwrap().insert("lava_strength".to_string(), serde_json::json!(self.lava_strength));
        config.params.as_object_mut().unwrap().insert("noise_strength".to_string(), serde_json::json!(self.noise_strength));
        config.params.as_object_mut().unwrap().insert("pitch_range".to_string(), serde_json::json!(self.pitch_range));
        
        // Spawn processing thread
        let (tx, rx) = channel();
        self.rx = Some(rx);
        let tx_clone = tx.clone();
        
        thread::spawn(move || {
             if let Err(e) = Self::run_preview_task(input_dir, output_dir, selected_actions, config, tx_clone) {
                 eprintln!("Preview Error: {}", e);
             }
        });
    }

    fn run_preview_task(input_dir: String, output_dir: String, actions: Vec<String>, config: ActionConfig, tx: Sender<AppMessage>) -> anyhow::Result<()> {
        let _ = tx.send(AppMessage::Log("🔍 寻找预览视频源...".to_string()));
         let video_files = Self::scan_video_files_static(&input_dir);
         
         if video_files.is_empty() {
             let _ = tx.send(AppMessage::Error("未找到视频文件，无法预览".to_string()));
             return Ok(());
         }
         
         let src_video = PathBuf::from(&video_files[0]);
         let _ = tx.send(AppMessage::Log(format!("📹 使用视频源: {:?}", src_video.file_name().unwrap_or_default())));
         
         // Setup directories
         let preview_dir = Path::new(&output_dir).join("preview");
         if !preview_dir.exists() {
             fs::create_dir_all(&preview_dir)?;
         }
         
         let preview_source = preview_dir.join("temp_source.mp4");
         
         // Step 1: Cut 5 seconds
         let _ = tx.send(AppMessage::Log("✂️ 正在截取前 5 秒...".to_string()));
         let ffmpeg_path = crate::core::ffutils::FFUtils::get_ffmpeg_path();
         
         let output = std::process::Command::new(&ffmpeg_path)
             .args(&[
                 "-y", "-ss", "0", "-t", "5", 
                 "-i", src_video.to_str().unwrap(),
                 "-c:v", "libx264", "-preset", "ultrafast", // Re-encode to ensure clean cut and compatibility
                 "-c:a", "aac",
                 preview_source.to_str().unwrap()
             ])
             .output()?;
             
         if !output.status.success() {
              let stderr = String::from_utf8_lossy(&output.stderr);
              let _ = tx.send(AppMessage::Error(format!("截取失败: {}", stderr)));
              return Ok(());
         }
         
         // Step 2: Apply actions (Chained)
         let _ = tx.send(AppMessage::Log("🚀 正在叠加应用所有效果...".to_string()));
         
         let mut current_input = preview_source.clone();
         let mut temp_files = Vec::new();
         
         for (i, action_id) in actions.iter().enumerate() {
             let _ = tx.send(AppMessage::Log(format!("  [{}/{}] 应用: {}", i+1, actions.len(), action_id)));
             
             // Reuse the static execution logic
             // Note: execute_action_static generates output based on input filename + action_id
             // We want to control the flow here.
             
             match Self::execute_action_static(action_id, &current_input, &preview_dir, &config) {
                 Ok(_) => {
                     // Determine the output path that execute_action_static created
                     let current_ext = current_input.extension().unwrap_or_default().to_string_lossy();
                     let current_stem = current_input.file_stem().unwrap_or_default().to_string_lossy();
                     
                     let expected_out_name = format!("{}_{}.{}", current_stem, action_id, current_ext);
                     let expected_out_path = preview_dir.join(&expected_out_name);
                     
                     if expected_out_path.exists() {
                         temp_files.push(current_input); // Mark previous as temp to delete
                         current_input = expected_out_path;
                     } else {
                          let _ = tx.send(AppMessage::Error(format!("Action {} finished but output not found", action_id)));
                          break;
                     }
                 },
                 Err(e) => {
                     let _ = tx.send(AppMessage::Error(format!("Action {} failed: {}", action_id, e)));
                     break;
                 }
             }
             
             // Update progress
             let _ = tx.send(AppMessage::Progress((i + 1) as f32 / actions.len() as f32));
         }
         
         // Step 3: Open Result
         let _ = tx.send(AppMessage::Log("✨ 预览生成完毕，正在打开...".to_string()));
         // Open the file using system default player
         #[cfg(target_os = "macos")]
         let _ = std::process::Command::new("open").arg(&current_input).spawn();
         
         #[cfg(target_os = "windows")]
         let _ = std::process::Command::new("cmd").args(&["/C", "start", "", current_input.to_str().unwrap()]).spawn();
         
         // Cleanup temps (optional: keep source for debug, but delete intermediate steps)
         // for p in temp_files { if p != preview_source { fs::remove_file(p).ok(); } }
         
         let _ = tx.send(AppMessage::Finished);
         Ok(())
    }
}

pub trait VideoMatrixUiExt {
    fn info_message(&mut self, text: &str);
}
impl VideoMatrixUiExt for egui::Ui {
    fn info_message(&mut self, text: &str) {
         self.label(egui::RichText::new(text).small().color(egui::Color32::GRAY));
    }
}

impl VideoMatrixApp {
    fn render_checkbox_group(&mut self, ui: &mut egui::Ui, title: &str, range: std::ops::Range<usize>, updates: &mut Vec<(String, String, bool, bool)>) {
        ui.heading(title);
        ui.add_space(5.0);
        
        egui::Grid::new(format!("grid_{}", title))
            .num_columns(4)
            .spacing([10.0, 10.0])
            .show(ui, |ui| {
                let mut col = 0;
                for i in range {
                    let (name, id, _checked) = &self.checkboxes[i];
                    let is_checked = self.selected_actions.contains(id);
                    let mut checked = is_checked;
                    
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut checked, name).changed() {
                            updates.push((id.clone(), name.clone(), is_checked, checked));
                        }
                        
                        // Add settings button for all actions
                        if ui.button("⚙").clicked() {
                            self.settings_action_id = id.clone();
                            self.show_settings_dialog = true;
                        }
                    });
                    
                    col += 1;
                    if col >= 4 { // 4 columns for better space usage
                        ui.end_row();
                        col = 0;
                    }
                }
                if col != 0 {
                    ui.end_row();
                }
            });
    }

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
        self.log("🚀 开始后台处理...");
        
        let input_dir = self.input_dir.clone();
        let output_dir = if self.output_dir.is_empty() {
            format!("{}/output", self.input_dir)
        } else {
            self.output_dir.clone()
        };
        let selected_actions = self.selected_actions.clone();
        let single_video_mode = self.single_video_mode;
        
        // Prepare config with material paths
        let mut config = ActionConfig::default();
        if !self.watermark_path.is_empty() { config.watermark_path = Some(self.watermark_path.clone()); }
        if !self.mask_path.is_empty() { config.mask_path = Some(self.mask_path.clone()); }
        if !self.sticker_path.is_empty() { config.sticker_path = Some(self.sticker_path.clone()); }
        if !self.border_path.is_empty() { config.border_path = Some(self.border_path.clone()); }
        if !self.light_effect_path.is_empty() { config.light_effect_path = Some(self.light_effect_path.clone()); }
        if !self.pip_path.is_empty() { config.pip_path = Some(self.pip_path.clone()); }
        if !self.goods_path.is_empty() { config.goods_path = Some(self.goods_path.clone()); }
        if !self.mask_video_path.is_empty() { config.mask_video_path = Some(self.mask_video_path.clone()); }
        
        // Add parameters
        config.params.as_object_mut().unwrap().insert("crop_min".to_string(), serde_json::json!(self.crop_min));
        config.params.as_object_mut().unwrap().insert("crop_max".to_string(), serde_json::json!(self.crop_max));
        config.params.as_object_mut().unwrap().insert("watermark_position".to_string(), serde_json::json!(self.watermark_position));
        config.params.as_object_mut().unwrap().insert("watermark_opacity".to_string(), serde_json::json!(self.watermark_opacity));
        
        // New parameters
        config.params.as_object_mut().unwrap().insert("rotate_angle".to_string(), serde_json::json!(self.rotate_angle));
        config.params.as_object_mut().unwrap().insert("speed_range".to_string(), serde_json::json!(self.speed_range));
        config.params.as_object_mut().unwrap().insert("target_fps".to_string(), serde_json::json!(self.target_fps));
        config.params.as_object_mut().unwrap().insert("target_bitrate".to_string(), serde_json::json!(self.target_bitrate));
        config.params.as_object_mut().unwrap().insert("sharpen_strength".to_string(), serde_json::json!(self.sharpen_strength));
        config.params.as_object_mut().unwrap().insert("denoise_strength".to_string(), serde_json::json!(self.denoise_strength));
        config.params.as_object_mut().unwrap().insert("blur_strength".to_string(), serde_json::json!(self.blur_strength));
        config.params.as_object_mut().unwrap().insert("grain_strength".to_string(), serde_json::json!(self.grain_strength));
        config.params.as_object_mut().unwrap().insert("vignette_strength".to_string(), serde_json::json!(self.vignette_strength));
        config.params.as_object_mut().unwrap().insert("border_width".to_string(), serde_json::json!(self.border_width));
        
        // Additional parameters
        config.params.as_object_mut().unwrap().insert("cut_seconds".to_string(), serde_json::json!(self.cut_seconds));
        config.params.as_object_mut().unwrap().insert("mirror_direction".to_string(), serde_json::json!(self.mirror_direction));
        config.params.as_object_mut().unwrap().insert("strong_crop_ratio".to_string(), serde_json::json!(self.strong_crop_ratio));
        config.params.as_object_mut().unwrap().insert("portrait_strength".to_string(), serde_json::json!(self.portrait_strength));
        config.params.as_object_mut().unwrap().insert("color_temp_range".to_string(), serde_json::json!(self.color_temp_range));
        config.params.as_object_mut().unwrap().insert("pull_width".to_string(), serde_json::json!(self.pull_width));
        config.params.as_object_mut().unwrap().insert("progressive_ratio".to_string(), serde_json::json!(self.progressive_ratio));
        config.params.as_object_mut().unwrap().insert("corner_radius".to_string(), serde_json::json!(self.corner_radius));
        config.params.as_object_mut().unwrap().insert("zoom_range".to_string(), serde_json::json!(self.zoom_range));
        config.params.as_object_mut().unwrap().insert("dissolve_strength".to_string(), serde_json::json!(self.dissolve_strength));
        config.params.as_object_mut().unwrap().insert("scan_strength".to_string(), serde_json::json!(self.scan_strength));
        config.params.as_object_mut().unwrap().insert("bounce_amplitude".to_string(), serde_json::json!(self.bounce_amplitude));
        config.params.as_object_mut().unwrap().insert("trifold_spacing".to_string(), serde_json::json!(self.trifold_spacing));
        config.params.as_object_mut().unwrap().insert("flash_strength".to_string(), serde_json::json!(self.flash_strength));
        config.params.as_object_mut().unwrap().insert("lava_strength".to_string(), serde_json::json!(self.lava_strength));
        config.params.as_object_mut().unwrap().insert("noise_strength".to_string(), serde_json::json!(self.noise_strength));
        config.params.as_object_mut().unwrap().insert("pitch_range".to_string(), serde_json::json!(self.pitch_range));
        
        // Create channel
        let (tx, rx) = channel();
        self.rx = Some(rx);
        
        // Clone for thread
        let tx_clone = tx.clone();
        
        // Spawn thread
        thread::spawn(move || {
            if let Err(e) = Self::process_thread(input_dir, output_dir, selected_actions, single_video_mode, config, tx_clone) {
                eprintln!("Thread error: {}", e);
            }
        });
    }

    fn process_thread(input_dir: String, output_dir: String, actions: Vec<String>, single_video_mode: bool, config: ActionConfig, tx: Sender<AppMessage>) -> anyhow::Result<()> {
        let _ = tx.send(AppMessage::Log(format!("📂 Input: {}", input_dir)));
        let _ = tx.send(AppMessage::Log(format!("📂 Output: {}", output_dir)));
        let _ = tx.send(AppMessage::Log(format!("✅ Selected {} features", actions.len())));
        let _ = tx.send(AppMessage::Log(format!("🎯 处理模式: {}", if single_video_mode { "单个视频功能叠加" } else { "每个功能独立输出" })));
        
        // Scan video files
        let _ = tx.send(AppMessage::Log("🔍 Scanning for video files...".to_string()));
        let video_files = Self::scan_video_files_static(&input_dir);
        
        if video_files.is_empty() {
            let _ = tx.send(AppMessage::Error("No video files found".to_string()));
            return Ok(());
        }
        
        let _ = tx.send(AppMessage::Log(format!("📹 Found {} video files", video_files.len())));
        let _ = tx.send(AppMessage::Log("🚀 正在使用多线程并行处理...".to_string()));
        
        let total_tasks = if single_video_mode {
            video_files.len() as f32
        } else {
            (video_files.len() * actions.len()) as f32
        };
        
        // Use AtomicUsize for thread-safe progress tracking
        let completed_tasks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        
        // Create output directory
        let out_path = PathBuf::from(&output_dir);
        if let Err(e) = fs::create_dir_all(&out_path) {
            let _ = tx.send(AppMessage::Error(format!("Failed to create output directory: {}", e)));
            return Ok(());
        }
        
        // Process video files in parallel using Rayon
        video_files.par_iter().for_each(|video_file| {
            let video_path = Path::new(video_file);
            let filename = video_path.file_name().unwrap().to_string_lossy();
            let tx = tx.clone(); // Clone sender for each thread
            
            if single_video_mode {
                // 单个视频叠加模式：所有动作按顺序应用到同一个视频
                let _ = tx.send(AppMessage::Log(format!("  ⏳ 叠加处理: {} [{}]...", filename, actions.join(" → "))));
                
                let mut current_input = video_path.to_path_buf();
                let mut temp_files = Vec::new();
                let mut success = true;
                
                for (i, action_id) in actions.iter().enumerate() {
                    let is_last_action = i == actions.len() - 1;
                    
                    let _ = tx.send(AppMessage::Log(format!("    [{}] 步骤 {}/{}: {}", filename, i + 1, actions.len(), action_id)));
                    
                    // 执行动作 - 动作会自动生成输出文件
                    let result = Self::execute_action_static(action_id, &current_input, &out_path, &config);
                    
                    match result {
                        Ok(_) => {
                            // 动作执行成功，现在需要找到生成的文件
                            // 动作会生成 {原文件名}_{动作名}.{扩展名} 格式的文件
                            
                            // 先保存当前的文件名信息（避免借用问题）
                            let current_ext = current_input.extension().and_then(|e| e.to_str()).unwrap_or("mp4").to_string();
                            let current_stem = current_input.file_stem().and_then(|s| s.to_str()).unwrap_or("video").to_string();
                            
                            // 如果是最后一个动作，使用_processed后缀
                            let output_filename = if is_last_action {
                                format!("{}_processed.{}", current_stem, current_ext)
                            } else {
                                format!("{}_{}.{}", current_stem, action_id, current_ext)
                            };
                            
                            let output_path = out_path.join(&output_filename);
                            
                            // 检查文件是否存在
                            if output_path.exists() {
                                if !is_last_action {
                                    temp_files.push(output_path.clone());
                                }
                                
                                // 如果是最后一个动作，重命名为_processed后缀
                                if is_last_action {
                                    let final_filename = format!("{}_processed.{}", current_stem, current_ext);
                                    let final_path = out_path.join(&final_filename);
                                    
                                    if let Err(e) = fs::rename(&output_path, &final_path) {
                                        let _ = tx.send(AppMessage::Log(format!("    [{}] ⚠️ 无法重命名为_processed: {}", filename, e)));
                                        current_input = output_path;
                                    } else {
                                        current_input = final_path;
                                        let _ = tx.send(AppMessage::Log(format!("    [{}] ✅ 已重命名为: {}", filename, final_filename)));
                                    }
                                } else {
                                    current_input = output_path;
                                }
                            } else {
                                // 如果标准命名不存在，尝试查找out_path中的新文件
                                // 注意：并行模式下 find_newest_video_file 基本不可靠，因为其他线程也在写入
                                // 所以我们只能尽量依赖 execute_action 返回准确的路径或者标准命名
                                // 这里我们假设动作实现是标准的，如果找不到文件，那就是出错了
                                // 但为了兼容之前的逻辑，我们还是保留这个Fallback，但要非常小心
                                // 实际上最好是让 execute return path. 但为了不改动太多 trait，我们先这样。
                                // 由于并行，find_newest_video_file 可能会找到别的线程产生的文件，这是个风险点。
                                // 简单的修复：execute_action 应该保证文件名。
                                // 我们前面已经看到了 VideoAction 只返回 Result<()>
                                // 不过我们的 get_dst 是确定的。
                                
                                let _ = tx.send(AppMessage::Log(format!("    [{}] ❌ 无法找到动作 {} 的输出文件 (标准命名不存在)", filename, action_id)));
                                success = false;
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppMessage::Log(format!("    [{}] ❌ {} 失败: {}", filename, action_id, e)));
                            success = false;
                            break;
                        }
                    }
                }
                
                // 清理临时文件
                for temp_file in temp_files {
                    let _ = fs::remove_file(temp_file);
                }
                
                // 更新进度
                let completed = completed_tasks.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                let _ = tx.send(AppMessage::Progress(completed as f32 / total_tasks));
                
                if success {
                    let _ = tx.send(AppMessage::Log(format!("  ✅ 叠加处理完成 ({})", filename)));
                } else {
                    let _ = tx.send(AppMessage::Log(format!("  ❌ 叠加处理失败 ({})", filename)));
                }
            } else {
                // 原始模式：每个动作生成独立视频
                for action_id in &actions {
                    let _ = tx.send(AppMessage::Log(format!("  ⏳ Processing: {} [{}]...", filename, action_id)));
                    
                    // Call corresponding action
                    let result = Self::execute_action_static(action_id, video_path, &out_path, &config);
                    
                    // 更新进度
                    let completed = completed_tasks.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    let _ = tx.send(AppMessage::Progress(completed as f32 / total_tasks));
                    
                    match result {
                        Ok(_) => {
                            let _ = tx.send(AppMessage::Log(format!("  ✅ {} Completed ({})", action_id, filename)));
                        }
                        Err(e) => {
                            let _ = tx.send(AppMessage::Log(format!("  ❌ {} Failed ({}): {}", action_id, filename, e)));
                        }
                    }
                }
            }
        });
        
        let _ = tx.send(AppMessage::Finished);
        Ok(())
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
    
    /// 查找输出目录中最新的视频文件（排除当前输入文件）
    fn find_newest_video_file(out_dir: &Path, current_input: &Path) -> Option<PathBuf> {
        let video_extensions = vec!["mp4", "mov", "mkv", "avi", "wmv", "flv", "webm", "m4v"];
        let mut newest_file: Option<PathBuf> = None;
        let mut newest_mtime: Option<std::time::SystemTime> = None;
        
        if let Ok(entries) = fs::read_dir(out_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        // 排除当前输入文件
                        if entry.path() == *current_input {
                            continue;
                        }
                        
                        // 检查文件扩展名
                        if let Some(ext) = entry.path().extension() {
                            if let Some(ext_str) = ext.to_str() {
                                if video_extensions.contains(&ext_str.to_lowercase().as_str()) {
                                    // 获取修改时间
                                    if let Ok(mtime) = metadata.modified() {
                                        if newest_mtime.is_none() || mtime > newest_mtime.unwrap() {
                                            newest_mtime = Some(mtime);
                                            newest_file = Some(entry.path());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        newest_file
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
            "mask_video" => MaskVideoAction.execute(src, out_dir, config),
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
            self.log("🛑 用户停止处理");
        }
    }
}

// Main function
pub fn run_desktop_app() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("视频矩阵 Pro v5.5.13"),
        ..Default::default()
    };
    
    eframe::run_native(
        "视频矩阵 Pro",
        options,
        Box::new(|cc| {
            // Load Chinese fonts
            let mut fonts = egui::FontDefinitions::default();
            
            // Try to load system fonts for Chinese support
            #[cfg(target_os = "macos")]
            let font_paths = vec![
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            ];
            
            #[cfg(target_os = "windows")]
            let font_paths = vec![
                "C:\\Windows\\Fonts\\msyh.ttc",
                "C:\\Windows\\Fonts\\simhei.ttf",
            ];
            
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            let font_paths: Vec<&str> = vec![];
            
            // Try loading fonts
            for path in font_paths {
                if let Ok(font_data) = std::fs::read(path) {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        Arc::new(egui::FontData::from_owned(font_data))
                    );
                    
                    // Insert at the beginning of all font families
                    fonts.families.entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, "chinese_font".to_owned());
                    
                    fonts.families.entry(egui::FontFamily::Monospace)
                        .or_default()
                        .insert(0, "chinese_font".to_owned());
                    
                    break; // Successfully loaded, stop trying
                }
            }
            
            cc.egui_ctx.set_fonts(fonts);
            
            Ok(Box::<VideoMatrixApp>::default())
        }),
    )
}