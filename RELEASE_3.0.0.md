# Aporia Loader v3.0.0

## 🎉 Complete Rewrite - Web Frontend Edition

### What's New

**🌐 Modern Web Interface**
- Beautiful dark-themed UI with cosmic background animation
- Real-time GitHub integration for releases and commits
- Dynamic version selector with automatic branch detection
- Smooth progress bar for downloads

**⚡ Performance Improvements**
- Lightweight Rust backend using wry webview
- Removed legacy GPU rendering code
- Optimized startup time
- Cross-platform support (Windows, Linux, macOS)

**🔧 Backend Enhancements**
- Proper JRE path detection (AppData\Roaming\apr)
- Correct Minecraft launch arguments with javaw.exe
- Automatic asset directory management
- Username and RAM configuration support

**📦 Build System**
- Automated GitHub Actions for all platforms
- Multi-platform binary releases (x86_64, ARM64)
- Automatic artifact uploads

### Technical Changes

- Migrated from Tauri to wry webview
- Removed old GPU rendering pipeline
- Cleaned up codebase (removed 1000+ lines of unused code)
- Implemented proper IPC communication between frontend and backend
- Added retry logic for GitHub API calls

### Platform Support

- ✅ Windows x64
- ✅ Linux x64
- ✅ macOS x64
- ✅ macOS ARM64

### Known Issues

- None at this time

### Installation

Download the appropriate binary for your platform from the releases page and run it.

### Credits

Built with Rust, wry, and modern web technologies.
