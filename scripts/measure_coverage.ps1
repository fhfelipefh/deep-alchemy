$ErrorActionPreference = "Stop"

$llvmBin = "$((rustc --print sysroot))\lib\rustlib\x86_64-pc-windows-msvc\bin"
$profdataExe = "$llvmBin\llvm-profdata.exe"
$covExe = "$llvmBin\llvm-cov.exe"

$covDir = "target\coverage"
if (Test-Path $covDir) {
    Remove-Item -Recurse -Force $covDir
}
New-Item -ItemType Directory -Force -Path $covDir | Out-Null

$env:RUSTFLAGS = "-C instrument-coverage"
$env:LLVM_PROFILE_FILE = "$covDir\cov-%p-%m.profraw"

Write-Host ">>> Running library unit tests with instrumentation coverage..."
cargo test --lib

Write-Host ">>> Merging raw profile data..."
$profrawFiles = Get-ChildItem -Path $covDir -Filter "*.profraw" | ForEach-Object { $_.FullName }
if ($profrawFiles.Count -eq 0) {
    Write-Error "No .profraw files generated!"
    exit 1
}

& $profdataExe merge -sparse $profrawFiles -o "$covDir\merged.profdata"

Write-Host ">>> Locating instrumented library binary..."
$libBin = (Get-ChildItem -Path "target\debug\deps" -Filter "deep_alchemy-*.exe" | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName

Write-Host "Using binary: $libBin"

Write-Host ">>> Generating Detailed Coverage Report for src/ ..."
& $covExe report $libBin `
    -instr-profile="$covDir\merged.profdata" `
    -ignore-filename-regex="\.cargo|rustc|tests|build|wgl" `
    -sources src\
