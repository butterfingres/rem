AC_INIT

AC_ARG_VAR([EMACS], [Emacs command])
AC_ARG_VAR([EMACSFLAGS], [Emacs command options])
AC_ARG_VAR([RUSTC], [Rust compiler command])
AC_ARG_VAR([RUSTFLAGS], [Rust compiler options])
AC_ARG_VAR([CARGO], [Cargo command])
AC_ARG_VAR([CARGOPROFILE], [Cargo profile [default: debug]])

AC_PROG_GREP
AC_PATH_PROG([EMACS], [${EMACS:-emacs}])
AC_SUBST([EMACS])
AC_SUBST([EMACSFLAGS])
AC_PATH_PROG([CARGO], [${CARGO:-cargo}])
AC_SUBST([CARGO])
AC_PATH_PROG([RUSTC], [${RUSTC:-rustc}])

AC_MSG_CHECKING([rust target tuple])
[RUST_TARGET_TUPLE="$(echo ${RUSTFLAGS} | awk 'match($0, /--target(=|[[:space:]]*)([^[:space:]]*)/, matches) { print matches[2] }')"]
AS_IF([test -z "${RUST_TARGET_TUPLE}"], [RUST_TARGET_TUPLE="$(${RUSTC} --print host-tuple)"])
AC_MSG_RESULT([${RUST_TARGET_TUPLE}])
AC_SUBST([RUST_TARGET_TUPLE])

AC_MSG_CHECKING([cargo profile])
AS_IF([test -z "${CARGOPROFILE}"], [CARGOPROFILE="debug"])
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
