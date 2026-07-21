use std::{
    borrow::Cow,
    env, fs,
    path::{Path, PathBuf},
    process::{Stdio, Command},
};

#[test]
fn lisp_test() {
    let lib_test_module_so = test_cdylib::build_current_project();

    let mut test_module_so = lib_test_module_so.clone();
    test_module_so.set_file_name("test-module");
    if let Some(ext) = lib_test_module_so.extension() {
        test_module_so.set_extension(ext);
    }

    if !test_module_so.exists() {
        fs::hard_link(lib_test_module_so, &test_module_so).unwrap();
    }

    assert!(
        Command::new(
            &*env::var_os("EMACS")
                .map(PathBuf::from)
                .map(Cow::Owned)
                .unwrap_or(Cow::Borrowed(Path::new("emacs"))),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env("EMACS_MODULE_RS_DEBUG", "1")
        .arg("-Q")
        .arg("-batch")
        .arg("-L")
        .arg(test_module_so.parent().unwrap())
        .arg("-L")
        .arg(env!("OUT_DIR"))
        .arg("-l")
        .arg("test-module-tests")
        .arg("-l")
        .arg("ert")
        .arg("-f")
        .arg("ert-run-tests-batch-and-exit")
        .status()
        .unwrap()
        .success()
    );
}
