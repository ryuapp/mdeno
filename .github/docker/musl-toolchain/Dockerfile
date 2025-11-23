# syntax=docker/dockerfile:1
# SPDX-License-Identifier: Apache-2.0 OR MIT
# https://github.com/taiki-e/rust-cross-toolchain

ARG UBUNTU_VERSION=24.04
ARG RUST_TARGET

# Use musl-cross-make to build toolchain
FROM ghcr.io/taiki-e/build-base:ubuntu-${UBUNTU_VERSION} AS builder
SHELL ["/bin/bash", "-CeEuxo", "pipefail", "-c"]
ARG DEBIAN_FRONTEND=noninteractive

ARG MUSL_CROSS_MAKE_REV=966bee4397acebc00dcb63754c0594ae91d08e9a
ARG BINUTILS_VERSION=2.40
ARG GCC_VERSION=13.3.0
ARG MUSL_VERSION=1.2.5
ARG LINUX_VERSION=headers-4.19.88-2

RUN mkdir -p -- /musl-cross-make
RUN curl --proto '=https' --tlsv1.2 -fsSL --retry 10 --retry-connrefused "https://github.com/taiki-e/musl-cross-make/archive/${MUSL_CROSS_MAKE_REV}.tar.gz" \
        | tar xzf - --strip-components 1 -C /musl-cross-make

ARG RUST_TARGET
ARG TOOLCHAIN_DIR="/${RUST_TARGET}"
RUN mkdir -p -- "${TOOLCHAIN_DIR}"

RUN <<EOF
case "${RUST_TARGET}" in
    x86_64-unknown-linux-musl) cc_target=x86_64-linux-musl ;;
    aarch64-unknown-linux-musl) cc_target=aarch64-linux-musl ;;
    *) echo "Unsupported target: ${RUST_TARGET}" && exit 1 ;;
esac
printf '%s\n' "${cc_target}" >/CC_TARGET
EOF

RUN <<EOF
cc_target=$(</CC_TARGET)
export CFLAGS="-fPIC -g1 ${CFLAGS:-}"
cd -- musl-cross-make
cat >./config.mak <<EOF2
OUTPUT = ${TOOLCHAIN_DIR}
TARGET = ${cc_target}
BINUTILS_VER = ${BINUTILS_VERSION}
GCC_VER = ${GCC_VERSION}
MUSL_VER = ${MUSL_VERSION}
LINUX_VER = ${LINUX_VERSION}
DL_CMD = curl -fsSL --retry 10 --retry-connrefused -C - -o
COMMON_CONFIG += CC="gcc -static --static" CXX="g++ -static --static"
COMMON_CONFIG += CFLAGS="-g1 -O2" CXXFLAGS="-g1 -O2" LDFLAGS="-s -static --static"
COMMON_CONFIG += --disable-nls
COMMON_CONFIG += --with-debug-prefix-map=\$(CURDIR)=
GCC_CONFIG += --enable-default-pie --enable-static-pie
GCC_CONFIG += --enable-languages=c,c++,fortran
GCC_CONFIG += --disable-libquadmath --disable-libquadmath-support --disable-decimal-float
GCC_CONFIG += --disable-multilib
EOF2

case "${RUST_TARGET}" in
    aarch64-*) common_config="--with-arch=armv8-a" ;;
esac
printf '%s\n' "${common_config:+"COMMON_CONFIG += ${common_config}"}" >>./config.mak
cat -- ./config.mak
make install -j"$(nproc)" &>build.log || (tail <build.log -5000 && exit 1)
EOF

RUN --mount=type=bind,source=./common.sh,target=/tmp/common.sh \
    bash /tmp/common.sh

# Fix ld-musl-*.so.1 symbolic link
RUN <<EOF
cc_target=$(</CC_TARGET)
case "${RUST_TARGET}" in
    aarch64-*) ldso_arch=aarch64 ;;
    x86_64*) ldso_arch=x86_64 ;;
    *) echo "Unsupported target: ${RUST_TARGET}" && exit 1 ;;
esac
cd -- "${TOOLCHAIN_DIR}/${cc_target}/lib"
ln -sf -- libc.so "ld-musl-${ldso_arch}.so.1"
EOF

FROM ubuntu:${UBUNTU_VERSION} AS final
SHELL ["/bin/bash", "-CeEuxo", "pipefail", "-c"]
ARG DEBIAN_FRONTEND=noninteractive
ARG RUST_TARGET
COPY --from=builder /"${RUST_TARGET}" /"${RUST_TARGET}"
ENV PATH="/${RUST_TARGET}/bin:$PATH"
