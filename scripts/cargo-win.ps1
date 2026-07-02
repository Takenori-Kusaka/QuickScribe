#Requires -Version 5
# QuickScribe: run cargo for src-tauri (Rust) locally with the same toolchain as CI (#467).
# Usage: powershell -File scripts/cargo-win.ps1 check   /   ... test --lib
# Why / setup: see docs/process/windows-local-rust-build.md
#   (needs MSVC link.exe + CMake policy<3.5 workaround + libclang 18 for bindgen).
param([Parameter(ValueFromRemainingArguments = $true)] [string[]] $CargoArgs)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent

$vswhere = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path $vswhere)) { throw "vswhere not found. Install VS Build Tools (C++). See docs/process/windows-local-rust-build.md" }
$vcvars = & $vswhere -latest -products * -find "VC\Auxiliary\Build\vcvars64.bat" | Select-Object -First 1
if (-not $vcvars) { throw "vcvars64.bat not found. Install the C++ workload of VS Build Tools." }

if (-not $env:LIBCLANG_PATH) { $env:LIBCLANG_PATH = "C:\libclang18\clang\native" }
$lib = $env:LIBCLANG_PATH
if (-not (Test-Path (Join-Path $lib "libclang.dll"))) {
  throw "libclang.dll not found in $lib . Run: python -m pip install --target C:\libclang18 libclang==18.1.1"
}

$argstr = $CargoArgs -join " "
$q = [char]34
$pathset = "set " + $q + "PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\CMake\bin;%PATH%" + $q
$cmd = "call " + $q + $vcvars + $q + " >nul 2>&1 && " + $pathset + " && set " + $q + "LIBCLANG_PATH=" + $lib + $q + " && set " + $q + "CMAKE_POLICY_VERSION_MINIMUM=3.5" + $q + " && cd /d " + $q + $root + "\src-tauri" + $q + " && cargo " + $argstr
& cmd.exe /c $cmd
exit $LASTEXITCODE
