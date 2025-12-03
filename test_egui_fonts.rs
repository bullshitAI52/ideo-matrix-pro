use eframe::egui;
use std::collections::BTreeMap;

struct TestApp {
    text: String,
}

impl Default for TestApp {
    fn default() -> Self {
        Self {
            text: "测试中文显示：Hello 世界！🚀".to_string(),
        }
    }
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("中文显示测试");
            
            ui.label("系统字体测试:");
            ui.label(&self.text);
            
            ui.separator();
            
            ui.label("测试字符串:");
            ui.label("✅ 全能去重面板");
            ui.label("✅ 后期增补功能");
            ui.label("✅ 一键MD5 (容器重封装)");
            ui.label("✅ 随机微裁切 (1-5%)");
            ui.label("✅ 浏览 保存至 立即执行 停止");
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("输入文本:");
                ui.text_edit_singleline(&mut self.text);
            });
            
            ui.separator();
            
            // 显示可用字体
            ui.collapsing("系统字体信息", |ui| {
                let fonts = ctx.fonts(|f| f.fonts_metadata());
                ui.label(format!("可用字体数量: {}", fonts.len()));
                
                for (i, font) in fonts.iter().enumerate().take(10) {
                    ui.label(format!("{}. {} - {}", i+1, font.family, font.font_name));
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 400.0])
            .with_title("egui 中文显示测试"),
        ..Default::default()
    };
    
    eframe::run_native(
        "egui 中文测试",
        options,
        Box::new(|_cc| Ok(Box::<TestApp>::default())),
    )
}