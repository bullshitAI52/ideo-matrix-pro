use eframe::egui;

struct SimpleFontTest {
    text: String,
}

impl Default for SimpleFontTest {
    fn default() -> Self {
        Self {
            text: "测试中文显示".to_string(),
        }
    }
}

impl eframe::App for SimpleFontTest {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 在应用启动时配置字体
        if !ctx.memory(|mem| mem.data.get_temp::<bool>("fonts_configured").unwrap_or(false)) {
            ctx.memory_mut(|mem| mem.data.insert_temp("fonts_configured", true));
            
            // 尝试配置中文字体
            let mut fonts = egui::FontDefinitions::default();
            
            // 使用系统默认字体
            fonts.families.insert(
                egui::FontFamily::Proportional,
                vec![
                    "PingFang SC".to_string(),
                    "Microsoft YaHei".to_string(),
                    "Noto Sans CJK SC".to_string(),
                    "Arial".to_string(),
                    "Helvetica".to_string(),
                ],
            );
            
            fonts.families.insert(
                egui::FontFamily::Monospace,
                vec![
                    "PingFang SC".to_string(),
                    "Microsoft YaHei".to_string(),
                    "Noto Sans CJK SC".to_string(),
                    "Courier New".to_string(),
                    "Monaco".to_string(),
                ],
            );
            
            ctx.set_fonts(fonts);
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("简单中文显示测试");
            
            ui.label("测试字符串:");
            ui.label("✅ 全能去重面板");
            ui.label("✅ 后期增补功能");
            ui.label("✅ 一键MD5 (容器重封装)");
            ui.label("✅ 随机微裁切 (1-5%)");
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("自定义文本:");
                ui.text_edit_singleline(&mut self.text);
            });
            
            ui.label(&format!("显示结果: {}", self.text));
        });
    }
}

pub fn run_simple_font_test() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_title("简单中文测试"),
        ..Default::default()
    };
    
    eframe::run_native(
        "简单中文测试",
        options,
        Box::new(|_cc| Ok(Box::<SimpleFontTest>::default())),
    )
}