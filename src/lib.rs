use clap::{self, ArgAction, Command, arg, value_parser};
use std::{
    error::Error,
    fs::{Permissions, create_dir, create_dir_all, set_permissions},
    os::unix::fs::PermissionsExt,
    path::Path,
    process,
    str::FromStr,
};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Default)]
struct Mode {
    user_read: bool,
    user_write: bool,
    user_execute: bool,
    group_read: bool,
    group_write: bool,
    group_execute: bool,
    other_read: bool,
    other_write: bool,
    other_execute: bool,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let str = s.to_lowercase();
        let mut mode = Mode::default();

        if str.is_empty() {
            return Err(format!("Mode must be defined"));
        }

        if str.contains("=") {
            for group_perms in s.split(",") {
                let (class, perms) = group_perms
                    .split_once("=")
                    .ok_or_else(|| format!("Invalid permission format: '{}'", group_perms))?;

                if perms.chars().any(|c| !"rwx".contains(c)) {
                    return Err(format!("Invalid permissions in: {}", group_perms));
                }

                for perm in perms.chars() {
                    match (class, perm) {
                        ("u", 'r') => mode.user_read = true,
                        ("u", 'w') => mode.user_write = true,
                        ("u", 'x') => mode.user_execute = true,
                        ("g", 'r') => mode.group_read = true,
                        ("g", 'w') => mode.group_write = true,
                        ("g", 'x') => mode.group_execute = true,
                        ("o", 'r') => mode.other_read = true,
                        ("o", 'w') => mode.other_write = true,
                        ("o", 'x') => mode.other_execute = true,
                        _ => return Err(format!("Unknown class or perm: {}={}", class, perm)),
                    }
                }
            }

            Ok(mode)
        } else {
            if s.chars().any(|c| !"rwx".contains(c)) {
                return Err(format!("Invalid mode: {}", s));
            }

            let read = s.contains('r');
            let write = s.contains('w');
            let exec = s.contains('x');

            Ok(Mode {
                user_read: read,
                user_write: write,
                user_execute: exec,
                group_read: read,
                group_write: write,
                group_execute: exec,
                other_read: read,
                other_write: write,
                other_execute: exec,
            })
        }
    }
}

impl From<&Mode> for Permissions {
    fn from(value: &Mode) -> Self {
        let mut bits = 0;

        if value.user_read {
            bits |= 0o400;
        }
        if value.user_write {
            bits |= 0o200;
        }
        if value.user_execute {
            bits |= 0o100;
        }
        if value.group_read {
            bits |= 0o040;
        }
        if value.group_write {
            bits |= 0o020;
        }
        if value.group_execute {
            bits |= 0o010;
        }
        if value.other_read {
            bits |= 0o004;
        }
        if value.other_write {
            bits |= 0o002;
        }
        if value.other_execute {
            bits |= 0o001;
        }

        PermissionsExt::from_mode(bits)
    }
}

#[derive(Debug)]
pub struct Config {
    dir_name: Vec<String>,
    parents: bool,
    verbose: bool,
    mode: Option<Mode>,
}

pub fn read_config() -> MyResult<Config> {
    let app = Command::new("mkdirr")
        .version("0.1.0")
        .author("Edgar Asatryan <easatryan2000@gmail.com>")
        .about("Rust mkdir")
        .args([
            arg!(<DIRECTORY> "Directory(ies)")
                .action(ArgAction::Append)
                .id("dir_name"),
            arg!(-p --parents "No error if existing, make parent directories as needed")
                .id("parents"),
            arg!(-v --verbose "Print a message for each created directory").id("verbose"),
            arg!(-m --mode <MODE> "Set file mode (read, write, execute)")
                .required(false)
                .value_parser(value_parser!(Mode))
                .id("mode"),
        ])
        .get_matches();

    let mode = app.get_one::<Mode>("mode").cloned();

    Ok(Config {
        dir_name: app
            .get_many::<String>("dir_name")
            .unwrap()
            .map(String::from)
            .collect::<Vec<String>>(),
        parents: app.get_flag("parents"),
        verbose: app.get_flag("verbose"),
        mode,
    })
}

fn create_directory(dir_name: &str, parents: bool, verbose: bool) -> MyResult<()> {
    let path = Path::new(dir_name);
    let mut verbose_info = String::new();

    if parents {
        if path.exists() {
            return Ok(());
        }

        if verbose {
            for ancestor in path.ancestors() {
                if ancestor.exists() || ancestor.as_os_str() == "" {
                    continue;
                }

                verbose_info.insert_str(
                    0,
                    format!("created directory '{}'\n", ancestor.display()).as_str(),
                );
            }
        }

        create_dir_all(path)?;

        if verbose && !verbose_info.is_empty() {
            print!("{}", verbose_info);
        }
        return Ok(());
    }

    create_dir(dir_name)?;
    if verbose {
        println!("created directory '{dir_name}'");
    }
    Ok(())
}

pub fn run(config: &Config) -> MyResult<()> {
    let mut exit_status = 0;
    for dir in config.dir_name.iter() {
        match create_directory(&dir, config.parents, config.verbose) {
            Err(e) => {
                exit_status = 1;
                eprintln!("cannot create directory `{dir}` {e}");
            }
            Ok(_) => {
                if let Some(mode) = &config.mode {
                    set_permissions(dir, mode.into())?;
                }
            }
        }
    }

    if exit_status == 1 {
        process::exit(exit_status);
    }

    Ok(())
}
