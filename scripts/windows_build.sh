#!/bin/bash

rm -rf windows_build/
cargo build --release
mkdir -p windows_build/

cp target/release/graphpu.exe windows_build/Graphpu.exe
cp -r resources windows_build/
cd windows_build
rcedit "Graphpu.exe" --set-icon "resources\\app_icon.ico"