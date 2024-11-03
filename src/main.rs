#[cfg(target_os = "windows")]
#[macro_use]
extern crate windows_service;

#[cfg(target_os = "windows")]
mod windows_impl {
    use std::ffi::OsString;
    use std::process::Command;
    use std::sync::Mutex;
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

    // Static storage to hold arguments passed to main
    lazy_static::lazy_static! {
        static ref SERVICE_ARGS: Mutex<Vec<OsString>> = Mutex::new(Vec::new());
    }

    fn my_service_main(arguments: Vec<OsString>) {
        let result: Result<()>;
        if arguments.len() > 1 {
            result = run_service(arguments); // Run the service with the arguments passed with the service start command if any
        } else {
            result = run_service(SERVICE_ARGS.lock().unwrap().clone()); // Else, run the service with the arguments passed with the service create command
        }
        if let Err(err) = result {
            eprintln!("Error starting the service: {:?}", err);
        }
    }

    fn run_service(arguments: Vec<OsString>) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let running_handle = running.clone();

        // Register the service control handler
        let status_handle =
            service_control_handler::register(SERVICE_NAME, move |control_event| {
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

        let shared_child = Arc::new(Mutex::new(None));
        let shared_child_clone = Arc::clone(&shared_child);
        let runing_clone = Arc::clone(&running);
        // Start the SSH command in a separate thread
        std::thread::spawn(move || {
            let args: Vec<String> = arguments
                .into_iter()
                .skip(1) // Skip the first argument which is the service name
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();

            if let Ok(child) = Command::new("ssh").args(args).spawn() {
                // Save the child process to be able to kill it later
                shared_child_clone.lock().unwrap().replace(child);
                // Wait until the service is stopped
                while runing_clone.load(Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_secs(1));
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
        let child = shared_child.lock().unwrap().take();
        if let Some(mut child) = child {
            if let Err(e) = child.kill() {
                eprintln!("Error killing the SSH process: {:?}", e);
            }
        }

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

    pub fn run_service_dispatcher() -> Result<()> {
        let args: Vec<OsString> = std::env::args_os().collect();
        // Store arguments in global storage
        *SERVICE_ARGS.lock().unwrap() = args.clone();

        service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn main() {
    if let Err(e) = windows_impl::run_service_dispatcher() {
        eprintln!("Error during the service: {:?}", e);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    panic!("This program is a Windows service and it can run only on Windows");
}
