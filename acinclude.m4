m4_define([RUST_TUPLE], [
  AC_MSG_CHECKING([rust target $1])
  rust_$2="$(${RUSTC} ${RUSTFLAGS} --print cfg | ${GREP} '^$2=' | cut -d \" -f 2)"
  AC_MSG_RESULT([${rust_$2}])
])

m4_define([CARGO_FILE_NAME], [
  AC_MSG_CHECKING([$1 file name])
  $2="$(${RUSTC} ${RUSTFLAGS} --crate-name=$3 --crate-type=cdylib --print=file-names $4/src/lib.rs)"
  AC_MSG_RESULT([${$2}])
])
