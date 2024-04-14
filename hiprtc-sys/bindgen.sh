#!/bin/bash

set -eux

# https://stackoverflow.com/questions/4774054/reliable-way-for-a-bash-script-to-get-the-full-path-to-itself
SCRIPTPATH="$(cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P)"

# The rocm include path may need to be adjusted
bindgen "${SCRIPTPATH}"/wrapper.h \
        --raw-line "#![allow(non_camel_case_types)]" \
        --raw-line "#![allow(non_upper_case_globals)]" \
        --raw-line "#![allow(non_snake_case)]" \
        --rustified-enum "hip.*" \
        --generate-block \
        --ctypes-prefix "::libc" \
        --with-derive-default \
        --with-derive-eq \
        --with-derive-ord \
        --with-derive-hash \
        -o "${SCRIPTPATH}"/src/bindings.rs \
        -- -I /opt/rocm/include/
