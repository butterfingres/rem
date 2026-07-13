AC_INIT

AC_ARG_VAR([RUSTC], [Rust compiler command])
AC_ARG_VAR([RUSTFLAGS], [Rust compiler options])
AC_ARG_VAR([CARGO], [Cargo command])
AC_ARG_VAR([CARGOPROFILE], [Cargo profile [default: release]])

AC_PROG_GREP
AC_PATH_PROG([CARGO], [${CARGO:-cargo}])
AC_PATH_PROG([RUSTC], [${RUSTC:-rustc}])

RUST_TUPLE([architecture], [target_arch])
RUST_TUPLE([vendor], [target_vendor])
RUST_TUPLE([operating system], [target_os])
RUST_TUPLE([environment], [target_env])

AC_MSG_CHECKING([rust target tuple])
RUST_TARGET_TUPLE=${rust_target_arch}-${rust_target_vendor}-${rust_target_os}-${rust_target_env}
AC_MSG_RESULT([${RUST_TARGET_TUPLE}])

AC_MSG_CHECKING([cargo profile])
AS_IF([test -z "${CARGOPROFILE}"], [CARGOPROFILE="release"])
AC_MSG_RESULT([${CARGOPROFILE}])

AC_MSG_CHECKING([cargo output directory])
CARGO_OUTPUT_DIR=target/${RUST_TARGET_TUPLE}/${CARGOPROFILE}
AC_MSG_RESULT([${CARGO_OUTPUT_DIR}])

CARGO_FILE_NAME([rs-module], [RS_MODULE], [rs_module], [rs-module])
CARGO_FILE_NAME([test-module], [TEST_MODULE], [test_module], [test-module])
CARGO_FILE_NAME([test-module-28], [TEST_MODULE_28], [test_module_28], [test-module-28])

AC_OUTPUT
