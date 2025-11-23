#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
# https://github.com/taiki-e/setup-cross-toolchain-action
set -CeEuo pipefail
IFS=$'\n\t'

x() {
  (
    set -x
    "$@"
  )
}

retry() {
  for i in {1..10}; do
    if "$@"; then
      return 0
    else
      sleep "${i}"
    fi
  done
  "$@"
}

bail() {
  printf '::error::%s\n' "$*"
  exit 1
}

_sudo() {
  if type -P sudo >/dev/null; then
    sudo "$@"
  else
    "$@"
  fi
}

export DEBIAN_FRONTEND=noninteractive

target="${INPUT_TARGET:?}"
host=$(rustc -vV | grep -E '^host:' | cut -d' ' -f2)
target_upper=$(echo "${target}" | tr '[:lower:]-' '[:upper:]_')
target_lower=$(echo "${target}" | tr '[:upper:]-' '[:lower:]_')

printf 'target: %s\n' "${target}"
printf 'host: %s\n' "${host}"

# Validate target
case "${target}" in
  x86_64-unknown-linux-musl|aarch64-unknown-linux-musl) ;;
  *) bail "unsupported target '${target}' (only x86_64-unknown-linux-musl and aarch64-unknown-linux-musl are supported)" ;;
esac

# Extract toolchain from Docker image (already built by docker/build-push-action)
printf '::group::Install toolchain\n'
toolchain_dir=/usr/local

retry docker create --name musl-toolchain-container musl-toolchain:latest
mkdir -p -- .setup-musl-toolchain-tmp
docker cp -- "musl-toolchain-container:/${target}" .setup-musl-toolchain-tmp/toolchain
docker rm -f -- musl-toolchain-container >/dev/null

_sudo cp -r -- .setup-musl-toolchain-tmp/toolchain/. "${toolchain_dir}"/
rm -rf -- ./.setup-musl-toolchain-tmp

sysroot_dir="${toolchain_dir}/${target}"

# Set environment variables for cross compilation
if type -P "${target}-gcc" >/dev/null; then
  cat >>"${GITHUB_ENV}" <<EOF
CARGO_TARGET_${target_upper}_LINKER=${target}-gcc
CC_${target_lower}=${target}-gcc
CXX_${target_lower}=${target}-g++
AR_${target_lower}=${target}-ar
RANLIB_${target_lower}=${target}-ranlib
CC=${target}-gcc
CXX=${target}-g++
STRIP=${target}-strip
OBJDUMP=${target}-objdump
PKG_CONFIG_PATH=/usr/lib/${target}/pkgconfig:${PKG_CONFIG_PATH:-}
PKG_CONFIG_SYSROOT_DIR=${sysroot_dir}
EOF
else
  bail "no suitable compiler found for target '${target}'"
fi

printf '::endgroup::\n'

# Verify installation
printf '::group::Verify toolchain\n'
x ${target}-gcc --version
x ${target}-g++ --version
printf '::endgroup::\n'
