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
    
    // Material Paths
    watermark_path: String,
    mask_path: String,
    sticker_path: String,
    border_path: String,
    light_effect_path: String,
    pip_path: String,
    goods_path: String,
    
    // Thread communication
    rx: Option<Receiver<AppMessage>>,
    
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
}

// Tab Enum
#[derive(PartialEq, Clone, Copy)]
enum Tab {
    All,       // All-in-One Panel
    Additional, // Additional Features
    Materials,  // New Materials Tab
    Help,      // Help & Documentation
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
            ("非线性变速 (0.9-1.1x)".to_string(), "speed".to_string(), false),
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
            current_tab: Tab::All,
            rx: None,
            log_messages: vec![
                "✨ 视频矩阵 Pro 已就绪".to_string(),
                "💡 提示：选择输入目录，勾选功能，然后点击\"开始处理\"".to_string(),
            ],
            checkboxes,
            watermark_path: String::new(),
            mask_path: String::new(),
            sticker_path: String::new(),
            border_path: String::new(),
            light_effect_path: String::new(),
            pip_path: String::new(),
            goods_path: String::new(),
            action_params: std::collections::HashMap::new(),
            show_settings_dialog: false,
            settings_action_id: String::new(),
            crop_min: 0.01,
            crop_max: 0.05,
            watermark_position: "top_right".to_string(),
            watermark_opacity: 0.5,
            
            // Defaults
            rotate_angle: 1.5,
            speed_range: 0.1,
            target_fps: 60,
            target_bitrate: "15M".to_string(),
            sharpen_strength: 1.0,
            denoise_strength: 5.0,
            blur_strength: 2.0,
            grain_strength: 0.1,
            vignette_strength: 0.5,
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
            pitch_range: 2.0,
        }
    }
}

impl eframe::App for VideoMatrixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // === Custom Visuals for Better Aesthetics ===
        let mut visuals = egui::Visuals::dark();
        
        // Grey Theme & High Contrast
        visuals.window_fill = egui::Color32::from_rgb(50, 50, 50); // Lighter grey background
        visuals.panel_fill = egui::Color32::from_rgb(50, 50, 50);
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
        
        // High contrast text
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
        
        visuals.selection.bg_fill = egui::Color32::from_rgb(100, 100, 100); // Grey selection
        ctx.set_visuals(visuals);

        // Increase Font Size
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)), // Base font size 16
            (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace)),
            (egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Small, egui::FontId::new(12.0, egui::FontFamily::Proportional)),
        ].into();
        ctx.set_style(style);

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
            // Header
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("视频矩阵 Pro").size(24.0).strong());
                ui.label(egui::RichText::new("V5.4").size(14.0).color(egui::Color32::GRAY));
            });
            ui.add_space(10.0);
            
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
                    
                    Tab::Help => {
                        ui.heading("📖 使用说明");
                        ui.add_space(10.0);
                        
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                ui.label("欢迎使用视频矩阵 Pro！本工具提供 51 种视频处理功能，帮助您快速批量处理视频。");
                                ui.add_space(10.0);
                                
                                // 基础使用
                                ui.heading("🚀 快速开始");
                                ui.label("1. 选择输入目录（包含要处理的视频文件）");
                                ui.label("2. 勾选需要的功能（可多选）");
                                ui.label("3. 点击功能旁的 ⚙️ 按钮调整参数（可选）");
                                ui.label("4. 点击\"开始处理\"按钮");
                                ui.label("5. 处理完成后，视频将保存在输出目录");
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
        
        // Prepare config with material paths
        let mut config = ActionConfig::default();
        if !self.watermark_path.is_empty() { config.watermark_path = Some(self.watermark_path.clone()); }
        if !self.mask_path.is_empty() { config.mask_path = Some(self.mask_path.clone()); }
        if !self.sticker_path.is_empty() { config.sticker_path = Some(self.sticker_path.clone()); }
        if !self.border_path.is_empty() { config.border_path = Some(self.border_path.clone()); }
        if !self.light_effect_path.is_empty() { config.light_effect_path = Some(self.light_effect_path.clone()); }
        if !self.pip_path.is_empty() { config.pip_path = Some(self.pip_path.clone()); }
        if !self.goods_path.is_empty() { config.goods_path = Some(self.goods_path.clone()); }
        
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
            if let Err(e) = Self::process_thread(input_dir, output_dir, selected_actions, config, tx_clone) {
                eprintln!("Thread error: {}", e);
            }
        });
    }

    fn process_thread(input_dir: String, output_dir: String, actions: Vec<String>, config: ActionConfig, tx: Sender<AppMessage>) -> anyhow::Result<()> {
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
            .with_title("视频矩阵 Pro V5.4"),
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