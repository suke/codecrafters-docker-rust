use anyhow::{Context, Result};
use std::io::{self, Write};
use std::process::{Command, Stdio};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = Command::new(command)
        .stderr(Stdio::inherit())
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    if output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
    } else {
        match output.status.code() {
            Some(code) => std::process::exit(code),
            None => {
                let stderr = io::stderr();
                let mut handle = stderr.lock();
                handle.write_all(b"Process terminated by signal")?;
            }
        }
    }

    Ok(())
}
