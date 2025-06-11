use clap::{self, ArgAction, Command, arg, value_parser};
use std::{
    error::Error,
    fs::{self, Permissions, create_dir, create_dir_all, set_permissions},
    os::unix::fs::PermissionsExt,
    str::FromStr,
};

pub type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
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

impl Default for Mode {
    fn default() -> Self {
        Self {
            user_read: false,
            user_write: false,
            user_execute: false,
            group_read: false,
            group_write: false,
            group_execute: false,
            other_read: false,
            other_write: false,
            other_execute: false,
        }
    }
}

impl TryFrom<String> for Mode {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let s = value.to_lowercase();
        let mut mode = Mode::default();

        if s.is_empty() {
            return Err(format!("Mode must be defined"));
        }

        if s.contains("=") {
            for group_perms in s.split(",") {
                let mut iter = group_perms.splitn(2, "=");
                let class = iter.next().unwrap();
                let perms = iter
                    .next()
                    .ok_or(format!("Invalid mode segment: {}", group_perms))?;

                if perms.chars().any(|c| !"rwx".contains(c)) {
                    return Err(format!("Invalid permissions in: {}", group_perms));
                }

                for p in perms.chars() {
                    match (class, p) {
                        ("u", 'r') => mode.user_read = true,
                        ("u", 'w') => mode.user_write = true,
                        ("u", 'x') => mode.user_execute = true,
                        ("g", 'r') => mode.group_read = true,
                        ("g", 'w') => mode.group_write = true,
                        ("g", 'x') => mode.group_execute = true,
                        ("o", 'r') => mode.other_read = true,
                        ("o", 'w') => mode.other_write = true,
                        ("o", 'x') => mode.other_execute = true,
                        _ => return Err(format!("Unknown class or perm: {}={}", class, p)),
                    }
                }
            }

            Ok(mode)
        } else {
            if s.chars().any(|c| !"rwx".contains(c)) {
                return Err(format!("Invalid mode: {}", value));
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

impl FromStr for Mode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Mode::try_from(s.to_string())
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
    let metadata = fs::metadata(dir_name);
    let mut verbose_info = String::new();

    if parents {
        if metadata.is_ok() {
            return Ok(());
        }

        if verbose {
            let parts = dir_name.split("/").collect::<Vec<&str>>();

            for (index, _) in parts.iter().enumerate() {
                let (left, _) = parts.split_at(index + 1);
                let full_path = left.join("/");
                let metadata = fs::metadata(&full_path);

                if metadata.is_ok() {
                    continue;
                }
                verbose_info.push_str(format!("created directory '{}'\n", full_path).as_str());
            }
        }

        create_dir_all(dir_name)?;

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
    for dir in config.dir_name.iter() {
        match create_directory(&dir, config.parents, config.verbose) {
            Err(e) => eprintln!("cannot create directory `{dir}` {e}"),
            Ok(_) => {
                if let Some(mode) = &config.mode {
                    set_permissions(dir, mode.into())?;
                }
            }
        }
    }
    Ok(())
}
