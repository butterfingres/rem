AC_INIT

AC_ARG_VAR([EMACS], [Emacs command])
AC_ARG_VAR([EMACSFLAGS], [Emacs command options])
AC_ARG_VAR([RUSTC], [Rust compiler command])
AC_ARG_VAR([RUSTFLAGS], [Rust compiler options])
AC_ARG_VAR([CARGO], [Cargo command])
AC_ARG_VAR([CARGOPROFILE], [Cargo profile [default: release]])

AC_PROG_GREP
AC_PATH_PROG([EMACS], [${EMACS:-emacs}])
AC_SUBST([EMACS])
AC_SUBST([EMACSFLAGS])
AC_PATH_PROG([CARGO], [${CARGO:-cargo}])
AC_SUBST([CARGO])
AC_PATH_PROG([RUSTC], [${RUSTC:-rustc}])

RUST_TUPLE([architecture], [target_arch])
RUST_TUPLE([vendor], [target_vendor])
RUST_TUPLE([operating system], [target_os])
RUST_TUPLE([environment], [target_env])
RUST_TUPLE([abi], [target_abi])

AC_MSG_CHECKING([rust target tuple])
RUST_TARGET_TUPLE=${rust_target_arch}-${rust_target_vendor}-${rust_target_os}-${rust_target_env}${rust_target_abi}
AC_MSG_RESULT([${RUST_TARGET_TUPLE}])
AC_SUBST([RUST_TARGET_TUPLE])

AC_MSG_CHECKING([cargo profile])
AS_IF([test -z "${CARGOPROFILE}"], [CARGOPROFILE="release"])
AC_MSG_RESULT([${CARGOPROFILE}])
AC_SUBST([CARGOPROFILE])

AC_MSG_CHECKING([cargo build flags])
CARGOFLAGS="--target=${RUST_TARGET_TUPLE}"
AS_IF([test "${CARGOPROFILE}" != "debug"], [CARGOFLAGS="${CARGOBUILDFLAGS} --profile=${CARGOPROFILE}"])
AC_MSG_RESULT([${CARGOFLAGS}])
AC_SUBST([CARGOFLAGS])

AC_MSG_CHECKING([cargo output directory])
CARGO_OUTPUT_DIR=target/${RUST_TARGET_TUPLE}/${CARGOPROFILE}
AC_MSG_RESULT([${CARGO_OUTPUT_DIR}])
AC_SUBST([CARGO_OUTPUT_DIR])

CARGO_FILE_NAME([rs-module], [RS_MODULE], [rs_module], [rs-module])
CARGO_FILE_NAME([test-module], [TEST_MODULE], [test_module], [test-module])
CARGO_FILE_NAME([test-module-28], [TEST_MODULE_28], [test_module_28], [test-module-28])

AC_MSG_CHECKING([dynamic library suffix])
SO_SUFFIX="${RS_MODULE#*.}"
AC_MSG_RESULT([${SO_SUFFIX}])
AC_SUBST([SO_SUFFIX])

AC_CONFIG_FILES([Makefile])
AC_OUTPUT
