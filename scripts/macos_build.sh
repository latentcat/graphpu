#!/bin/bash

set -e

APP_NAME=graphpu
MACOS_BIN_NAME=GraphPU_bin
MACOS_APP_NAME=GraphPU
MACOS_APP_DIR=$MACOS_APP_NAME.app

MACOS_DMG_NAME=GraphPU_Installer
MACOS_DMG_DIR=$MACOS_DMG_NAME.dmg

mkdir -p macos_build
cd macos_build

echo "Creating app directory structure"
rm -rf $MACOS_APP_NAME
rm -rf $MACOS_APP_DIR
mkdir -p $MACOS_APP_DIR/Contents/MacOS

cargo rustc \
    --verbose \
    --release

echo "Copying binary"
MACOS_APP_BIN=$MACOS_APP_DIR/Contents/MacOS/$MACOS_BIN_NAME
cp ../target/release/$APP_NAME $MACOS_APP_BIN
chmod 755 $MACOS_APP_BIN

echo "Copying resources directory"
cp -r ../resources $MACOS_APP_DIR/Contents/MacOS

echo "Copying launcher"
cp ../scripts/macos_launch.sh $MACOS_APP_DIR/Contents/MacOS/$MACOS_APP_NAME
chmod 755 $MACOS_APP_DIR/Contents/MacOS/$MACOS_APP_NAME

echo "Copying Icon"
mkdir -p $MACOS_APP_DIR/Contents/Resources
cp ../resources/Info.plist $MACOS_APP_DIR/Contents/
cp ../resources/app.icns $MACOS_APP_DIR/Contents/Resources/

echo "Code Signing"
codesign -s "5BW8DTZV3H" --deep -v -f -o runtime $MACOS_APP_DIR

echo "Creating dmg"
# mkdir -p $MACOS_APP_NAME
# cp -r $MACOS_APP_DIR $MACOS_APP_NAME/
# rm -rf $MACOS_APP_NAME/.Trashes

# FULL_NAME=$MACOS_APP_NAME

# hdiutil create $FULL_NAME.dmg -srcfolder $MACOS_APP_NAME -ov
# rm -rf $MACOS_APP_NAME


# Since create-dmg does not clobber, be sure to delete previous DMG
[[ -f $MACOS_DMG_NAME.dmg ]] && rm $MACOS_DMG_NAME.dmg

# Create the DMG
create-dmg \
  --volname $MACOS_DMG_NAME \
  --volicon "../resources/app.icns" \
  --background "../scripts/installer_background.jpg" \
  --window-pos 200 120 \
  --window-size 540 375 \
  --icon-size 120 \
  --hide-extension $MACOS_APP_DIR \
  --text-size 12 \
  --icon $MACOS_APP_DIR 140 190 \
  --hide-extension $MACOS_APP_DIR \
  --app-drop-link 400 190 \
  $MACOS_DMG_NAME.dmg \
  "./"


xcrun notarytool submit $MACOS_DMG_NAME.dmg \
    --keychain-profile "nhciao" \
    --wait

xcrun stapler staple $MACOS_DMG_NAME.dmg