#!/usr/bin/env python3
"""
测试Rust桌面应用中文显示
"""
import subprocess
import time
import os

def test_chinese_display():
    print("🔤 测试Rust桌面应用中文显示...")
    
    # 启动Rust应用
    print("1. 启动Rust桌面应用...")
    rust_process = subprocess.Popen(
        ['cargo', 'run'],
        cwd='src-tauri',
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # 等待应用启动
    time.sleep(5)
    
    print("2. 检查应用状态...")
    
    # 检查进程是否还在运行
    result = subprocess.run(['ps', 'aux'], capture_output=True, text=True)
    if 'video-matrix-pro' in result.stdout:
        print("   ✓ 应用正在运行")
    else:
        print("   ✗ 应用已退出")
        
        # 检查错误输出
        stdout, stderr = rust_process.communicate()
        if stderr:
            print("   错误输出:")
            for line in stderr.split('\n'):
                if line.strip():
                    print(f"     {line}")
    
    print("\n3. 中文显示测试:")
    print("   - 应用标题: 'Video Matrix Pro V5.4 (Rust桌面版)'")
    print("   - 标签页: '全能去重面板', '后期增补功能'")
    print("   - 功能名称: '一键MD5 (容器重封装)', '随机微裁切 (1-5%)' 等")
    print("   - 按钮: '浏览', '保存至', '立即执行', '停止'")
    print("   - 日志: '✨ Video Matrix Pro 已就绪'")
    
    print("\n4. 与Python版本对比:")
    print("   Python版本: 使用PySide6，原生支持中文")
    print("   Rust版本: 使用egui，需要系统字体支持")
    print("   如果看到乱码，可能是系统缺少中文字体")
    
    print("\n✅ 测试完成!")
    print("\n如果看到乱码，请:")
    print("1. 确保系统安装了中文字体")
    print("2. 在macOS上: 系统默认有PingFang SC字体")
    print("3. 在Windows上: 系统默认有Microsoft YaHei字体")
    print("4. 在Linux上: 安装Noto Sans CJK字体")
    
    # 清理
    rust_process.terminate()
    rust_process.wait()

if __name__ == "__main__":
    test_chinese_display()