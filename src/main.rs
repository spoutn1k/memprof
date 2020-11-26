use nix::unistd::{execvp, fork, ForkResult, Pid};
use signal_hook::{register, SIGCHLD, SIGINT};
use std::ffi::CString;
use std::{env, process, thread, time};

use memprof::memory::peek;
use memprof::plot::plot;
use memprof::store::{format_run, Store};

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

    if args.len() < 2 {
        eprintln!("Usage: {} <command> <arguments>", args[0]);
        process::exit(1);
    }

    let store = Store::setup(".memprof".into()).expect("Error accessing cache");

    if args[1] == "--list" {
        if let Some(data) = store.list() {
            data.iter()
                .enumerate()
                .map(|(i, r)| format_run(i, r))
                .for_each(drop);
        }

        process::exit(0);
    }

    if args[1] == "--show" {
        let selected: usize;
        if args.len() > 2 {
            selected = args[2].parse::<usize>().unwrap();
        } else {
            selected = store.list().unwrap().len() - 1;
        }

        if let Some(mut profile) = store.get_profile(selected as u32) {
            plot(&mut profile);
        }

        process::exit(0);
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

    let mut profile = store.create_record(&args);

    while unsafe { !EXITED } {
        if let Ok(data) = peek(child_pid) {
            profile.record(data);
        }

        thread::sleep(granularity);
    }

    eprintln!("Child {} ran for {}ms", child_pid, profile.elapsed);
}
