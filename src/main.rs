use anyhow::Result;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use wry::WebViewBuilder;

mod github;
mod downloader;
mod launcher;

fn main() -> Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Aporia Loader v0.5.0")
        .with_inner_size(LogicalSize::new(1400.0, 800.0))
        .build(&event_loop)?;

    let html_path = std::env::current_dir()?.join("index.html");
    let html_content = std::fs::read_to_string(&html_path)?;

    log::info!("HTML loaded, size: {} bytes", html_content.len());

    let _webview = WebViewBuilder::new()
        .with_html(html_content)
        .with_devtools(true)
        .with_ipc_handler(|msg| {
            log::info!("IPC Message: {}", msg.body());
            let parts: Vec<&str> = msg.body().split(':').collect();
            if parts.is_empty() {
                return;
            }

            match parts[0] {
                "log" => {
                    if parts.len() > 1 {
                        log::info!("[JS] {}", parts[1..].join(":"));
                    }
                }
                "error" => {
                    if parts.len() > 1 {
                        log::error!("[JS] {}", parts[1..].join(":"));
                    }
                }
                "get_releases" => {
                    std::thread::spawn(move || {
                        tokio::runtime::Runtime::new().unwrap().block_on(async {
                            match github::GitHubClient::new() {
                                Ok(client) => {
                                    match client.get_releases().await {
                                        Ok(releases) => {
                                            log::info!("Releases loaded: {} items", releases.len());
                                        }
                                        Err(e) => log::error!("Failed to get releases: {}", e),
                                    }
                                }
                                Err(e) => log::error!("Failed to create GitHub client: {}", e),
                            }
                        });
                    });
                }
                "get_commits" => {
                    if parts.len() < 2 {
                        return;
                    }
                    let branch = parts[1].to_string();
                    std::thread::spawn(move || {
                        tokio::runtime::Runtime::new().unwrap().block_on(async move {
                            match github::GitHubClient::new() {
                                Ok(client) => {
                                    match client.get_commits(&branch).await {
                                        Ok(commits) => {
                                            log::info!("Commits loaded: {} items from {}", commits.len(), branch);
                                        }
                                        Err(e) => log::error!("Failed to get commits: {}", e),
                                    }
                                }
                                Err(e) => log::error!("Failed to create GitHub client: {}", e),
                            }
                        });
                    });
                }
                "download_version" => {
                    if parts.len() < 2 {
                        return;
                    }
                    let version = parts[1].to_string();
                    log::info!("Download version: {}", version);
                    std::thread::spawn(move || {
                        tokio::runtime::Runtime::new().unwrap().block_on(async move {
                            let downloader = downloader::Downloader::new();
                            match downloader.download_version(&version).await {
                                Ok(_) => log::info!("Downloaded version {}", version),
                                Err(e) => log::error!("Failed to download version: {}", e),
                            }
                            // Download assets after version
                            match downloader.download_assets(&version).await {
                                Ok(_) => log::info!("Downloaded assets for version {}", version),
                                Err(e) => log::error!("Failed to download assets: {}", e),
                            }
                        });
                    });
                }
                "download_jre" => {
                    log::info!("Download JRE");
                    std::thread::spawn(move || {
                        tokio::runtime::Runtime::new().unwrap().block_on(async {
                            let downloader = downloader::Downloader::new();
                            match downloader.download_jre().await {
                                Ok(_) => log::info!("Downloaded JRE"),
                                Err(e) => log::error!("Failed to download JRE: {}", e),
                            }
                        });
                    });
                }
                "launch" => {
                    if parts.len() < 2 {
                        return;
                    }
                    let version = parts[1].to_string();
                    log::info!("Launch version: {}", version);
                    std::thread::spawn(move || {
                        let launcher = launcher::Launcher::new();
                        match launcher.launch(&version) {
                            Ok(_) => log::info!("Launched version {}", version),
                            Err(e) => log::error!("Failed to launch: {}", e),
                        }
                    });
                }
                _ => {
                    log::warn!("Unknown IPC message: {}", msg.body());
                }
            }
        })
        .build(&window)?;

    log::info!("WebView created successfully");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                log::info!("Close requested");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
