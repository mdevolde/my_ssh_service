#[macro_use]
extern crate windows_service;

use std::ffi::OsString;
use std::process::Command;
use std::sync::atomic::AtomicU32;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use windows_service::{
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher, Result,
};

define_windows_service!(ffi_service_main, my_service_main);

const SERVICE_NAME: &str = "mysshservice";

fn my_service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service(arguments) {
        eprintln!("Service failed: {:?}", e);
    }
}

fn run_service(arguments: Vec<OsString>) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let running_handle = running.clone();

    // Register the service control handler
    let status_handle = service_control_handler::register(SERVICE_NAME, move |control_event| {
        match control_event {
            ServiceControl::Stop => {
                running_handle.store(false, Ordering::SeqCst); // Tell to the thread to stop when there is a stop event
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    })?;

    // Update the service status to Running
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    })?;

    let child_pid = Arc::new(AtomicU32::new(0));
    let child_pid_clone = Arc::clone(&child_pid);
    // Start the SSH command in a separate thread
    std::thread::spawn(move || {
        let args: Vec<String> = arguments
            .into_iter()
            .skip(1) // Skip the first argument which is the service name
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        if let Ok(mut child) = Command::new("ssh").args(args).spawn() {
            // Save the child process ID
            child_pid_clone.store(child.id(), Ordering::SeqCst);
            // Wait until the service is stopped
            if let Err(e) = child.wait() {
                eprintln!("Error waiting for the SSH command: {:?}", e);
            }
        } else {
            eprintln!("Error starting the SSH command");
        }
    });

    // Wait until the service is stopped
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // Stop the SSH process
    let child_pid = child_pid.load(Ordering::SeqCst);
    Command::new("taskkill")
        .args(&["/F", "/T", "/PID", &child_pid.to_string()])
        .output()
        .expect("Failed to kill the SSH process");

    // Update the service status to Stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn main() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}
