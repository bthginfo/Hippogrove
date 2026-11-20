#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
if ! command -v xcrun >/dev/null; then
    printf 'Für das iOS-Paket werden macOS und Xcode benötigt.\n' >&2
    exit 1
fi

if [[ ${HIPPOGROVE_IOS_DEVICE:-0} == 1 ]]; then
    sdk=iphoneos
    target=aarch64-apple-ios
    triple=arm64-apple-ios15.0
else
    sdk=iphonesimulator
    case "$(uname -m)" in
        arm64) target=aarch64-apple-ios-sim; triple=arm64-apple-ios15.0-simulator ;;
        x86_64) target=x86_64-apple-ios; triple=x86_64-apple-ios15.0-simulator ;;
        *) printf 'Unbekannte Mac-Architektur.\n' >&2; exit 1 ;;
    esac
fi

export SDKROOT="$(xcrun --sdk "$sdk" --show-sdk-path)"
export IPHONEOS_DEPLOYMENT_TARGET=15.0
out="$PWD/build/ios/$sdk"
app="$out/Hippogrove.app"
mkdir -p "$app/assets"
xcrun --sdk "$sdk" clang -x objective-c -target "$triple" -isysroot "$SDKROOT" -fobjc-arc \
    -c packaging/ios/Controller.m -o "$out/Controller.o"
xcrun libtool -static -o "$out/libHippogroveController.a" "$out/Controller.o"
RUSTFLAGS="${RUSTFLAGS:-} -Lnative=$out" cargo build --release --locked --no-default-features \
    --bin hippogrove --target "$target"
cp "${CARGO_TARGET_DIR:-target}/$target/release/hippogrove" "$app/hippogrove"
cp packaging/ios/Info.plist "$app/Info.plist"
cp -R assets/. "$app/assets/"
if [[ "$sdk" == iphonesimulator ]]; then
    codesign --force --sign - "$app"
fi
printf '%s\n' "$app"
