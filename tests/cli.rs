use assert_cmd::Command;
use predicates::prelude::*;
use rand::{Rng, distr::Alphanumeric, rng};
use regex::escape;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};
use tempfile::TempDir;

const PRG: &str = "mkdirr";

fn random_name() -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect()
}

#[test]
fn usage() -> Result<(), Box<dyn std::error::Error>> {
    for flag in &["-h", "--help"] {
        Command::cargo_bin(PRG)?
            .arg(flag)
            .assert()
            .stdout(predicate::str::contains("Usage"));
    }
    Ok(())
}

#[test]
fn success_with_one_param() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?.arg(&dir).assert().success();

    assert!(dir.is_dir());
    Ok(())
}

#[test]
fn success_with_multiple_param() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let names: Vec<String> = (0..4).map(|_| random_name()).collect();
    let paths: Vec<PathBuf> = names.iter().map(|n| tmp.path().join(n)).collect();

    Command::cargo_bin(PRG)?.args(&paths).assert().success();

    for p in &paths {
        assert!(p.is_dir());
    }
    Ok(())
}

#[test]
fn success_with_parents_flag() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let parent = tmp.path().join(random_name());
    let child = parent.join(random_name());

    Command::cargo_bin(PRG)?
        .args([child.to_str().unwrap(), "-p"])
        .assert()
        .success();

    assert!(child.is_dir());
    Ok(())
}

#[test]
fn success_with_parents_flag_when_dir_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let parent = tmp.path().join(random_name());
    let child = parent.join(random_name());

    Command::cargo_bin(PRG)?
        .args([child.to_str().unwrap(), "-p"])
        .assert()
        .success();

    Command::cargo_bin(PRG)?
        .args([child.to_str().unwrap(), "-p"])
        .assert()
        .success();

    assert!(child.is_dir());
    Ok(())
}

#[test]
fn success_with_multiple_params_and_parents_flag() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let parents: Vec<String> = (0..4).map(|_| random_name()).collect();
    let dir_names: Vec<PathBuf> = parents
        .iter()
        .map(|p| tmp.path().join(p).join(random_name()))
        .collect();

    Command::cargo_bin(PRG)?
        .args(&dir_names)
        .arg("-p")
        .assert()
        .success();

    for p in &dir_names {
        assert!(p.is_dir());
    }
    Ok(())
}

#[test]
fn success_with_verbose_flag() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());
    let expected = format!(
        r"created directory '{}'
",
        dir.display()
    );

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-v"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(&escape(&expected))?);

    assert!(dir.is_dir());
    Ok(())
}

#[test]
fn success_with_mode_option() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=r"])
        .assert()
        .success();

    assert!(dir.is_dir());
    Ok(())
}

#[test]
fn change_mode_if_directory_exists_and_parents_flag_provided()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?.arg(&dir).assert().success();
    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=w", "-p"])
        .assert()
        .success();

    let mode = fs::metadata(&dir)?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o222);
    Ok(())
}

#[test]
fn test_mode_all_rwx() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=rwx"])
        .assert()
        .success();

    let mode = fs::metadata(&dir)?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o777);
    Ok(())
}

#[test]
fn test_mode_user_rwx_group_rx_other_r() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=u=rwx,g=rx,o=r"])
        .assert()
        .success();

    let mode = fs::metadata(&dir)?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o754);
    Ok(())
}

#[test]
fn test_mode_user_rw_group_w_other_x() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=u=rw,g=w,o=x"])
        .assert()
        .success();

    let mode = fs::metadata(&dir)?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o621);
    Ok(())
}

#[test]
fn test_mode_only_user_rwx() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=u=rwx"])
        .assert()
        .success();

    let mode = fs::metadata(&dir)?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
    Ok(())
}

#[test]
fn test_mode_only_group_rx() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=g=rx"])
        .assert()
        .success();

    let mode = fs::metadata(&dir)?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o050);
    Ok(())
}

#[test]
fn test_mode_only_other_r() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=o=r"])
        .assert()
        .success();

    let mode = fs::metadata(&dir)?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o004);
    Ok(())
}

#[test]
fn fails_with_empty_parameters() -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin(PRG)?
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage: mkdirr"));
    Ok(())
}

#[test]
fn fails_when_directory_already_exists() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());
    let expected = format!(
        r"cannot create directory `{}` File exists \(os error 17\)\n?",
        escape(dir.to_str().unwrap())
    );

    Command::cargo_bin(PRG)?.arg(&dir).assert().success();
    Command::cargo_bin(PRG)?
        .arg(&dir)
        .assert()
        .stderr(predicate::str::is_match(&expected)?);

    Ok(())
}

#[test]
fn fails_when_param_contains_multiple_directories_with_no_parents_flag()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp
        .path()
        .join(random_name())
        .join(random_name())
        .join(random_name());
    let expected = format!(
        r"cannot create directory `{}` No such file or directory \(os error 2\)\n?",
        escape(dir.to_str().unwrap())
    );

    Command::cargo_bin(PRG)?
        .arg(&dir)
        .assert()
        .stderr(predicate::str::is_match(&expected)?);
    Ok(())
}

#[test]
fn fails_when_mode_option_is_empty() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());
    let expected = r"error: invalid value '' for '--mode <MODE>': Mode must be defined\n?";

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m="])
        .assert()
        .failure()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

#[test]
fn fails_when_mode_is_not_valid() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let dir = tmp.path().join(random_name());
    let expected = r"error: invalid value 'c' for '--mode <MODE>': Invalid mode: c\n?";

    Command::cargo_bin(PRG)?
        .args([dir.to_str().unwrap(), "-m=c"])
        .assert()
        .failure()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}
