mod desktop_app;
mod core;
mod actions;

fn main() {
    // 运行真正的桌面应用（非浏览器）
    if let Err(e) = desktop_app::run_desktop_app() {
        eprintln!("应用启动失败: {}", e);
    }
}
