use clap::{arg, Parser};
use std::fs;
use std::io::{self, Write};
use std::os::unix::net::UnixStream;

#[derive(Parser)]
#[command(name = "throw")]
#[command(author, version, about = "Send file paths and actions to the catcher.", long_about = None)]
#[command(arg_required_else_help = true)]
struct Args {
    /// Files to send (use --copy for files to copy instead of moving)
    #[arg(required = true)]
    files: Vec<String>,
  
    /// Execute tasks quietly
    #[arg(short, long)]
    quiet: bool,

    /// Copy files instead of moving
    #[arg(short, long)]
    copy: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Connect to the catcher via Unix Domain Socket
    let socket_path = "/tmp/yeetyeetyeet";
    match UnixStream::connect(socket_path) {
        Ok(mut stream) => {
            for filename in args.files {
                match fs::canonicalize(&filename) {
                    Ok(absolute_path) => {
                        // Prepare the command to send (copy or move)
                        let action = if args.copy { "copy" } else { "move" };
                        let message = format!("{}|{}\n", action, absolute_path.to_string_lossy());

                        // Send the message to the catcher
                        stream.write_all(message.as_bytes())?;
                        if !args.quiet {
                          println!("Sent: {} ({})", absolute_path.display(), action);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: Could not resolve '{}': {}", filename, e);
                    }
                }
            }
        }
        Err(_) => {
            eprintln!("Error: Could not connect to the catcher. Make sure the catcher is running.");
        }
    }

    Ok(())
}
