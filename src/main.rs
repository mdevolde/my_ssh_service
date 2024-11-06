#[cfg(target_os = "windows")]
mod service;

#[cfg(target_os = "windows")]
fn main() {
    // Create a log file in the directory %ProgramData%\mysshservice\logs
    let base_log_dir = std::env::var("ProgramData").unwrap();
    let log_dir: std::path::PathBuf =
        std::path::PathBuf::from(base_log_dir).join(format!("{}/logs", service::SERVICE_NAME));
    std::fs::create_dir_all(&log_dir).unwrap();

    // Initialize the logger
    let file_spec = flexi_logger::FileSpec::default().directory(log_dir);
    flexi_logger::Logger::with(flexi_logger::LevelFilter::Info) // Minimum log level to write
        .log_to_file(file_spec)
        .rotate(
            flexi_logger::Criterion::Size(10 * 1024 * 1024), // Rotate the log file when it reaches 10 MB
            flexi_logger::Naming::Numbers,
            flexi_logger::Cleanup::KeepLogFiles(7), // Keep the last 7 log files
        )
        // Log format: level [timestamp] message
        .format(|write, now, record| {
            write.write_fmt(format_args!(
                "{:<5} [{}] {}",
                record.level(),
                now.format("%Y-%m-%d %H:%M:%S"),
                record.args()
            ))
        })
        .start()
        .unwrap();

    // Run the service dispatcher to start the service
    if let Err(e) = service::run_service_dispatcher() {
        log::error!("Error during the service: {:?}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // Stop the compilation if the target OS is not Windows
    compile_error!("This program is a Windows service and it can run only on Windows");
}
