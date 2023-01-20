#!/bin/sh

echo "running zipalign"
$ANDROID_HOME/build-tools/30.0.3/zipalign -p 4 ../src-tauri/gen/android/ambient_lamp/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk ./ambient-lamp.apk

echo "signing apk"
$ANDROID_HOME/build-tools/30.0.3/apksigner sign --ks my.keystore ambient-lamp.apk

echo "veryfying apk sign"
$ANDROID_HOME/build-tools/30.0.3/apksigner verify ambient-lamp.apk