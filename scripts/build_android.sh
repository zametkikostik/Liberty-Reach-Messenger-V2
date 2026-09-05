#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../mobile"
flutter pub get
flutter build apk --release
echo "APK: build/app/outputs/flutter-apk/"
