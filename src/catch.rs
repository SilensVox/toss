use std::os::unix::net::UnixListener;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use clap::Parser;
use std::sync::Arc;
use std::thread;

#[derive(Parser)]
#[command(name = "catch")]
#[command(author, version, about = "Receives file paths and actions from the thrower.", long_about = None)]
struct Args {
    /// Run as a server (keep running after processing files)
    #[arg(short, long)]
    server: bool,

    /// Set the destination directory for the files
    destination: Option<String>,
}

fn handle_connection(
    stream: std::os::unix::net::UnixStream,
    destination_path: Arc<PathBuf>,
) -> io::Result<()> {
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                let mut parts = line.splitn(2, '|');
                let action = parts.next().unwrap_or_default();
                let source_path = parts.next().unwrap_or_default();

                let source = Path::new(source_path);
                let destination = destination_path.join(
                    source.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown")),
                );

                let command = match action {
                    "copy" => "cp",
                    "move" => "mv",
                    _ => {
                        eprintln!("Invalid action: {}", action);
                        continue;
                    }
                };

                let output = Command::new(command)
                    .arg(source_path)
                    .arg(&destination)
                    .output();

                match output {
                    Ok(output) => {
                        if !output.status.success() {
                            eprintln!(
                                "Failed to {} {}: {}",
                                command,
                                source_path,
                                String::from_utf8_lossy(&output.stderr)
                            );
                        } else {
                            println!(
                                "{} '{}' -> '{}'",
                                if command == "cp" { "Copied" } else { "Moved" },
                                source_path,
                                destination.display()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Error executing {} command: {}", command, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let socket_path = "/tmp/yeetyeetyeet";

    let destination_path: PathBuf = if let Some(dest) = &args.destination {
        let path = Path::new(dest);
        if !path.is_dir() {
            eprintln!("Error: '{}' is not a valid directory.", dest);
            std::process::exit(1);
        }
        path.to_path_buf()
    } else {
        std::env::current_dir()?
    };

    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    println!("Waiting...");

    let destination_path = Arc::new(destination_path);

    if args.server {
        // Infinite loop for server mode
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let destination_path = Arc::clone(&destination_path);
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, destination_path) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    } else {
        // Single connection handling for non-server mode
        match listener.accept() {
            Ok((stream, _)) => {
                let destination_path = Arc::clone(&destination_path);
                let handle = thread::spawn(move || {
                    handle_connection(stream, destination_path).unwrap_or_else(|e| {
                        eprintln!("Error handling connection: {}", e);
                    });
                });

                handle.join().expect("Thread panicked");
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }

    Ok(())
}
