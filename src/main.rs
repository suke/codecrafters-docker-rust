use anyhow::{Context, Result};
use std::env;
use std::io::{self, Write};
use std::os::unix::fs;
use std::path::Path;
use std::process::{Command, Stdio};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let dir = env::temp_dir();
    copy_executable_in_directory(&Path::new(command), dir.as_path())?;
    create_dev_null(&dir)?;

    fs::chroot(&dir)?;
    std::env::set_current_dir("/")?;

    #[cfg(target_os = "linux")]
    unsafe {
        libc::unshare(libc::CLONE_NEWPID);
    }

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
            Some(code) => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(&output.stdout)?;
                std::process::exit(code)
            }
            None => {
                let stderr = io::stderr();
                let mut handle = stderr.lock();
                handle.write_all(b"Process terminated by signal")?;
            }
        }
    }

    Ok(())
}

fn copy_executable_in_directory(executable_path: &Path, chroot_dir: &Path) -> Result<()> {
    let striped_prefix_path = executable_path.strip_prefix("/")?;
    let executable_path_inside_dir = chroot_dir.join(striped_prefix_path);

    std::fs::create_dir_all(&executable_path_inside_dir.parent().unwrap())?;
    std::fs::copy(executable_path, &executable_path_inside_dir)?;
    Ok(())
}

fn create_dev_null(chroot_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(chroot_dir.join("dev"))?;
    std::fs::File::create(chroot_dir.join("dev/null"))?;
    Ok(())
}
