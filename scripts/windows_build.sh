#!/bin/bash

cd ../

BUILD_DIR=windows_build
APP_BUNDLE_NAME=graphpu-0.4.5-windows-x86_64
APP_BUNDLE_DIR=${BUILD_DIR}/${APP_BUNDLE_NAME}
ARCH=x86_64-pc-windows-msvc

rm -rf ${APP_BUNDLE_DIR}
rustup target add ${ARCH}
cargo build --release --features exe --target ${ARCH}
mkdir -p ${APP_BUNDLE_DIR}/

cp target/${ARCH}/release/graphpu.exe ${APP_BUNDLE_DIR}/GraphPU.exe
cp -r resources ${APP_BUNDLE_DIR}/
#cd windows_build
#rcedit "Graphpu.exe" --set-icon "resources\\app_icon.ico"