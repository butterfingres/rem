use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() {
    let test_orig = Path::new("lisp").join("test-module-tests.el");
    println!("cargo::rerun-if-changed={}", test_orig.display());

    let test_copy =
        env::var_os("OUT_DIR").map(PathBuf::from).unwrap().join(test_orig.file_name().unwrap());
    fs::hard_link(&test_orig, &test_copy).unwrap();
    assert!(
        Command::new("emacs")
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .arg("-Q")
            .arg("-batch")
            .arg("-l")
            .arg("bytecomp")
            .arg("-f")
            .arg("batch-byte-compile")
            .arg(&test_copy)
            .status()
            .unwrap()
            .success()
    );
}
