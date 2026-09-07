#!/usr/bin/env python3
"""Build the real Android engine, bundled UI and APK (no Android Studio required)."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
NDK_VERSION = "27.2.12479018"
TARGETS = {"arm64-v8a": "aarch64-linux-android", "x86_64": "x86_64-linux-android"}


def run(args, env, cwd=ROOT):
    subprocess.run([str(arg) for arg in args], cwd=cwd, env=env, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--abi", action="append", choices=TARGETS, help="Repeat for multiple ABIs; default: both 64-bit ABIs")
    parser.add_argument("--release", action="store_true", help="Build an unsigned release APK; debug is installable with a local debug key")
    parser.add_argument("--skip-prepare", action="store_true", help="Reuse an already prepared, patched EasyTier source tree")
    args = parser.parse_args()
    env = os.environ.copy()
    sdk = Path(env.get("ANDROID_HOME") or env.get("ANDROID_SDK_ROOT") or Path.home() / "Library/Android/sdk")
    ndk = Path(env.get("ANDROID_NDK_HOME") or sdk / "ndk" / NDK_VERSION)
    env["ANDROID_HOME"] = env["ANDROID_SDK_ROOT"] = str(sdk)
    if not env.get("JAVA_HOME"):
        homebrew_java = Path("/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home")
        if homebrew_java.is_dir():
            env["JAVA_HOME"] = str(homebrew_java)
    if env.get("JAVA_HOME"):
        env["PATH"] = str(Path(env["JAVA_HOME"]) / "bin") + os.pathsep + env["PATH"]
    hosts = {"Darwin": "darwin-x86_64", "Linux": "linux-x86_64", "Windows": "windows-x86_64"}
    host = hosts.get(platform.system())
    if host is None:
        raise SystemExit("Unsupported build host")
    toolchain = ndk / "toolchains/llvm/prebuilt" / host / "bin"
    if not toolchain.is_dir():
        raise SystemExit(f"Install Android NDK {NDK_VERSION} with sdkmanager, or set ANDROID_NDK_HOME")
    if not (sdk / "platforms/android-35/android.jar").is_file():
        raise SystemExit("Install platforms;android-35 and build-tools;35.0.0 with sdkmanager")
    if "PROTOC" not in env and not shutil.which("protoc", path=env["PATH"]):
        cached = ROOT / ".cache/protoc-31.1/bin/protoc"
        if not cached.is_file():
            raise SystemExit("Install a protobuf compiler and set PROTOC to its executable")
        env["PROTOC"] = str(cached)
    if not args.skip_prepare:
        run(["python3", "scripts/prepare-engine.py"], env)
    elif not (ROOT / ".cache/quicklan-easytier/easytier/src/quicklan_policy.rs").is_file():
        raise SystemExit("Patched EasyTier source is absent; omit --skip-prepare")
    abis = list(dict.fromkeys(args.abi or TARGETS))
    libraries = ROOT / "android/app/src/main/jniLibs"
    libraries.mkdir(parents=True, exist_ok=True)
    # This directory is generated only by this script. Prevent a stale, unrequested
    # ABI from silently shipping alongside the currently built libraries.
    for abi in TARGETS:
        if abi not in abis and (libraries / abi).exists():
            shutil.rmtree(libraries / abi)
    for abi in abis:
        target = TARGETS[abi]
        target_env = env.copy()
        suffix = ".cmd" if platform.system() == "Windows" else ""
        clang = toolchain / f"{target}26-clang{suffix}"
        clangxx = toolchain / f"{target}26-clang++{suffix}"
        ar = toolchain / ("llvm-ar.exe" if platform.system() == "Windows" else "llvm-ar")
        key = target.replace("-", "_")
        target_env[f"CARGO_TARGET_{key.upper()}_LINKER"] = str(clang)
        target_env[f"CC_{key}"] = str(clang)
        target_env[f"CXX_{key}"] = str(clangxx)
        target_env[f"AR_{key}"] = str(ar)
        target_env[f"CARGO_TARGET_{key.upper()}_RUSTFLAGS"] = "-C link-arg=-Wl,-z,max-page-size=16384"
        run(["cargo", "build", "--manifest-path", "android/native/Cargo.toml", "--target", target, "--release", "--locked"], target_env)
        source = ROOT / "android/native/target" / target / "release/libquicklan_android.so"
        (libraries / abi).mkdir(exist_ok=True)
        shutil.copy2(source, libraries / abi / source.name)
    npm = "npm.cmd" if platform.system() == "Windows" else "npm"
    run([npm, "run", "build", "--", "--base=/assets/"], env)
    assets = ROOT / "android/app/src/main/assets"
    assets.mkdir(parents=True, exist_ok=True)
    # All assets are generated. Preserve neither old hashed JS nor old notices.
    for child in assets.iterdir():
        if child.is_dir():
            shutil.rmtree(child)
        else:
            child.unlink()
    shutil.copytree(ROOT / "dist", assets, dirs_exist_ok=True)
    notices = assets / "licenses"
    notices.mkdir()
    run(["python3", "scripts/generate-notices.py", "--android"], env)
    for source in [ROOT / "THIRD_PARTY_NOTICES", ROOT / "LICENSE", ROOT / "licenses/GPL-3.0.txt", ROOT / "licenses/EasyTier-LGPL-3.0.txt", ROOT / "licenses/ANDROID_DEPENDENCY_LICENSES.txt", ROOT / "licenses/ANDROID_PLATFORM_NOTICES.txt"]:
        shutil.copy2(source, notices / source.name)
    wrapper = ROOT / "android" / ("gradlew.bat" if platform.system() == "Windows" else "gradlew")
    variant = "release" if args.release else "debug"
    run([wrapper, "--no-daemon", f"assemble{variant.title()}"], env, cwd=ROOT / "android")
    name = "app-release-unsigned.apk" if args.release else "app-debug.apk"
    apk = ROOT / "android/app/build/outputs/apk" / variant / name
    version = json.loads((ROOT / "package.json").read_text())["version"]
    output = ROOT / "artifacts/android"
    output.mkdir(parents=True, exist_ok=True)
    destination = output / f"QuickLAN-{version}-android-{variant}.apk"
    shutil.copy2(apk, destination)
    report = {"apk": str(destination.relative_to(ROOT)), "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(), "abis": abis, "variant": variant, "ndk": NDK_VERSION, "min_sdk": 26, "target_sdk": 35, "signing": "unsigned" if args.release else "local debug key"}
    (output / f"build-{variant}.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
