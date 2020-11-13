use nix::unistd::{execvp, fork, ForkResult, Pid};
use signal_hook::{register, SIGCHLD};
use std::ffi::CString;
use std::io::{self, BufRead, Write};
use std::{env, fs, process, thread, time};

static mut EXITED: bool = false;

fn handle_sigchild() {
    unsafe { EXITED = true };
}

fn child_method(path: &String, args: &[String]) {
    fn convert(data: &String) -> CString {
        CString::new(data.as_bytes()).expect("Failed converting String to C-like string")
    }

    let c_args: Vec<CString> = args.iter().map(|x| convert(x)).collect();

    execvp(&convert(path), &c_args).expect("Error launching {}");
}

fn extract_number(line: &String, prefix: &str) -> Option<u64> {
    line.strip_prefix(prefix)?
        .strip_suffix("kB")?
        .trim()
        .parse::<u64>()
        .ok()
}

struct Entry {
    v_size: u64,
    v_peak: u64,
    r_size: u64,
    r_peak: u64,
}

fn main() {
    let granularity = time::Duration::from_millis(1);
    let args: Vec<String> = env::args().collect();
    let child_pid: Pid;
    let now: time::Instant;

    if args.len() < 2 {
        eprintln!("Usage: {} <command> <arguments>", args[0]);
        process::exit(1);
    }

    unsafe { register(SIGCHLD, handle_sigchild).expect("Error registering signal") };

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            child_pid = child;
        }

        Ok(ForkResult::Child) => {
            child_pid = Pid::from_raw(0);
            child_method(&args[1], &args[1..]);
        }

        Err(_) => {
            println!("Fork failed");
            process::exit(1);
        }
    }

    let procfile = format!("/proc/{}/status", child_pid);
    let outfile = format!("/tmp/memprof.tsv");
    now = time::Instant::now();

    let mut outfile =
        fs::File::create(&outfile).expect(&format!("Error opening file {}", &outfile));

    outfile
        .write_all(
            "Real Size (kB)\tReal Peak (kB)\tVirtual Size (kB)\tVirtual Peak (kB)\n".as_bytes(),
        )
        .expect("Could not write to file");

    while unsafe { !EXITED } {
        let procfile =
            fs::File::open(&procfile).expect(&format!("Error opening file {}", &procfile));

        let mut data = Entry {
            v_size: 0,
            v_peak: 0,
            r_size: 0,
            r_peak: 0,
        };

        for result in io::BufReader::new(procfile).lines() {
            if let Ok(line) = result {
                if line.contains("VmRSS") {
                    match extract_number(&line, "VmRSS:\t") {
                        Some(value) => data.r_size = value,
                        None => {}
                    }
                }

                if line.contains("VmHWM") {
                    match extract_number(&line, "VmHWM:\t") {
                        Some(value) => data.r_peak = value,
                        None => {}
                    }
                }

                if line.contains("VmSize") {
                    match extract_number(&line, "VmSize:\t") {
                        Some(value) => data.v_size = value,
                        None => {}
                    }
                }

                if line.contains("VmPeak") {
                    match extract_number(&line, "VmPeak:\t") {
                        Some(value) => data.v_peak = value,
                        None => {}
                    }
                }
            }
        }

        outfile
            .write_all(
                format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    now.elapsed().as_secs_f32(),
                    data.r_size,
                    data.r_peak,
                    data.v_size,
                    data.v_peak
                )
                .as_bytes(),
            )
            .expect("Could not write to file");

        thread::sleep(granularity);
    }

    eprintln!(
        "Child {} ran for {}ms",
        child_pid,
        now.elapsed().as_millis()
    );
}
