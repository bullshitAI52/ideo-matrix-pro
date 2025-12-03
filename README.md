# Video Matrix Pro (Rust Desktop)

A high-performance video processing tool built with Rust and Tauri.

## Prerequisites

- **Node.js**: Required for the frontend build.
- **Rust**: Required for the backend build.
- **FFmpeg**: Must be installed and available in your system PATH.

## How to Run (Development Mode)

To run the application in development mode (with hot-reloading):

1.  Open your terminal (Terminal.app or iTerm2).
2.  Navigate to the project directory:
    ```bash
    cd /Users/apple/Downloads/video_processor
    ```
3.  Install dependencies (first time only):
    ```bash
    npm install
    ```
4.  Run the application:
    ```bash
    npm run tauri dev
    ```

## How to Build (Release Version)

To build a standalone `.app` or `.dmg` file for distribution:

1.  Run the build command:
    ```bash
    npm run tauri build
    ```
2.  The executable will be located in:
    `src-tauri/target/release/bundle/macos/`

## Building for Windows

Since this application uses native system libraries, **cross-compiling from macOS to Windows is not recommended** and is very difficult to set up.

### Option 1: Build on a Windows PC (Recommended)
1.  Copy this project to a Windows computer.
2.  Install [Rust](https://www.rust-lang.org/tools/install) and [Node.js](https://nodejs.org/).
3.  Install **Microsoft Visual Studio C++ Build Tools**.
4.  Run `npm install` and `npm run tauri build`.
5.  The `.exe` and `.msi` installers will be generated in `src-tauri/target/release/bundle/msi/`.

### Option 2: Use GitHub Actions (Automated)
You can set up a GitHub Action to automatically build the Windows version whenever you push code.
1.  Create `.github/workflows/build.yml`.
2.  Use the standard Tauri build action matrix to build for `windows-latest`.

## Troubleshooting

-   **"Start Processing" button is disabled**: Make sure you have selected an **Input Directory** and checked at least one **Feature**.
-   **Garbled Text**: The app uses system fonts to support Chinese characters in file paths. If you see issues, ensure you are on a standard macOS installation.
