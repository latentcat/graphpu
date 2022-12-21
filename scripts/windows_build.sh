#!/bin/bash

cd ../

BUILD_DIR=windows_build
APP_BUNDLE_NAME=graphpu-0.4.0-windows-x86_64
APP_BUNDLE_DIR=${BUILD_DIR}/${APP_BUNDLE_NAME}

rm -rf ${APP_BUNDLE_DIR}
cargo build --release --features exe
mkdir -p ${APP_BUNDLE_DIR}/

cp target/release/graphpu.exe ${APP_BUNDLE_DIR}/GraphPU.exe
cp -r resources ${APP_BUNDLE_DIR}/
#cd windows_build
#rcedit "Graphpu.exe" --set-icon "resources\\app_icon.ico"