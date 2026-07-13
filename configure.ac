AC_INIT

AC_ARG_VAR([RUSTC], [Rust compiler command])
AC_ARG_VAR([RUSTFLAGS], [Rust compiler options])
AC_ARG_VAR([CARGO], [Cargo command])
AC_ARG_VAR([CARGOPROFILE], [Cargo profile [default: release]])

AC_PROG_GREP
AC_PATH_PROG([CARGO], [${CARGO:-cargo}])
AC_PATH_PROG([RUSTC], [${RUSTC:-rustc}])

AC_MSG_CHECKING([rust target architecture])
rust_arch="$(${RUSTC} ${RUSTFLAGS} --print cfg | ${GREP} '^target_arch=' | cut -d \" -f 2)"
AC_MSG_RESULT([${rust_arch}])

AC_MSG_CHECKING([rust target vendor])
rust_vendor="$(${RUSTC} ${RUSTFLAGS} --print cfg | ${GREP} '^target_vendor=' | cut -d \" -f 2)"
AC_MSG_RESULT([${rust_vendor}])

AC_MSG_CHECKING([rust target os])
rust_os="$(${RUSTC} ${RUSTFLAGS} --print cfg | ${GREP} '^target_os=' | cut -d \" -f 2)"
AC_MSG_RESULT([${rust_os}])

AC_MSG_CHECKING([rust target environment])
rust_env="$(${RUSTC} ${RUSTFLAGS} --print cfg | ${GREP} '^target_env=' | cut -d \" -f 2)"
AC_MSG_RESULT([${rust_env}])

AC_MSG_CHECKING([rust target tuple])
RUST_TARGET_TUPLE=${rust_arch}-${rust_vendor}-${rust_os}-${rust_env}
AC_MSG_RESULT([${RUST_TARGET_TUPLE}])

AC_MSG_CHECKING([cargo profile])
AS_IF([test -z "${CARGOPROFILE}"], [CARGOPROFILE="release"])
AC_MSG_RESULT([${CARGOPROFILE}])

AC_MSG_CHECKING([cargo output directory])
CARGO_OUTPUT_DIR=target/${RUST_TARGET_TUPLE}/${CARGOPROFILE}
AC_MSG_RESULT([${CARGO_OUTPUT_DIR}])

AC_OUTPUT
