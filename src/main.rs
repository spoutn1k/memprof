use nix::unistd::{execvp, fork, ForkResult, Pid};
use signal_hook::{register, SIGCHLD, SIGINT};
use std::ffi::CString;
use std::io::Write;
use std::{env, fs, process, thread, time};

use memprof::memory::peek;
use memprof::store::{setup_store, Store};

static mut EXITED: bool = false;

fn handle_sigchild() {
    unsafe { EXITED = true };
}

fn child_method(args: &[String]) {
    fn convert(data: &String) -> CString {
        CString::new(data.as_bytes()).expect("Failed converting String to C-like string")
    }

    let c_args: Vec<CString> = args.iter().map(|x| convert(x)).collect();

    execvp(&convert(&args[0]), &c_args).expect("Error launching {}");
}

fn main() {
    let granularity = time::Duration::from_millis(1);
    let args: Vec<String> = env::args().collect();
    let child_pid: Pid;
    let now: time::Instant;

    let mut store = Store::new();

    if args.len() < 2 {
        eprintln!("Usage: {} <command> <arguments>", args[0]);
        process::exit(1);
    }

    if let Err(e) = setup_store(&mut store) {
        eprintln!("Error accessing cache: {}", e);
        process::exit(1)
    }

    unsafe {
        register(SIGCHLD, handle_sigchild).expect("Error registering SIGCHLD");
        register(SIGINT, handle_sigchild).expect("Error registering SIGINT");

        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                child_pid = child;
            }

            Ok(ForkResult::Child) => {
                child_pid = Pid::from_raw(0);
                child_method(&args[1..]);
            }

            Err(_) => {
                println!("Fork failed");
                process::exit(1);
            }
        }
    }

    let outfile = store.create_record(&args);
    now = time::Instant::now();

    let mut outfile = fs::File::create(&outfile).unwrap();

    outfile
        .write_all(
            "Real Size (kB)\tReal Peak (kB)\tVirtual Size (kB)\tVirtual Peak (kB)\n".as_bytes(),
        )
        .expect("Could not write to file");

    while unsafe { !EXITED } {
        if let Ok(data) = peek(child_pid) {
            outfile
                .write_all(
                    format!(
                        "{}\t{}\t{}\t{}\t{}\n",
                        now.elapsed().as_secs_f32(),
                        data.r_size(),
                        data.r_peak(),
                        data.v_size(),
                        data.v_peak()
                    )
                    .as_bytes(),
                )
                .expect("Could not write to file");
        }

        thread::sleep(granularity);
    }

    eprintln!(
        "Child {} ran for {}ms",
        child_pid,
        now.elapsed().as_millis()
    );
}
