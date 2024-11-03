#[cfg(target_os = "windows")]
mod service;

#[cfg(target_os = "windows")]
fn main() {
    // Run the service dispatcher to start the service
    if let Err(e) = service::run_service_dispatcher() {
        eprintln!("Error during the service: {:?}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // Stop the compilation if the target OS is not Windows
    compile_error!("This program is a Windows service and it can run only on Windows");
}
