# Builds the prototypes with the MSVC toolchain and measures them.
param(
    [ValidateSet('tauri', 'sw', 'fv', 'sk')][string[]]$Only = @('tauri', 'sw', 'fv', 'sk'),
    [ValidateSet('full', 'present')][string]$Mode = 'full',
    [switch]$Maximized,
    [string]$TargetRoot = (Join-Path $PSScriptRoot 'target')
)
$ErrorActionPreference = 'Stop'
$thumbs = Join-Path $PSScriptRoot 'thumbs'
if (-not (Test-Path (Join-Path $thumbs 't999.jpg'))) { throw 'Run make-thumbs.ps1 first.' }

$saved = @{}
foreach ($n in 'RUSTUP_TOOLCHAIN', 'BENCH_PRESENT', 'BENCH_COLS') { $saved[$n] = [Environment]::GetEnvironmentVariable($n) }
try {
    $env:RUSTUP_TOOLCHAIN = 'stable-x86_64-pc-windows-msvc'
    $features = @{ sw = 'software'; fv = 'femtovg'; sk = 'skia' }

    function Invoke-Build([string]$crate, [string]$target, [string[]]$extra) {
        $manifest = Join-Path $PSScriptRoot "$crate/Cargo.toml"
        & cargo build --release --locked --manifest-path $manifest --target-dir (Join-Path $TargetRoot $target) @extra
        if ($LASTEXITCODE -ne 0) { throw "Build failed: $target" }
    }

    Invoke-Build 'harness' 'harness' @()
    foreach ($v in $Only) {
        if ($v -eq 'tauri') { Invoke-Build 'tauri-app' 'tauri' @() }
        else { Invoke-Build 'slint-app' "slint-$v" @('--features', $features[$v]) }
    }

    # An empty value removes the variable for the child process.
    $env:BENCH_PRESENT = if ($Mode -eq 'present') { '1' } else { $null }
    $env:BENCH_COLS = if ($Maximized) { '8' } else { $null }
    $harness = Join-Path $TargetRoot 'harness/release/harness.exe'
    foreach ($v in $Only) {
        $exe = if ($v -eq 'tauri') { Join-Path $TargetRoot 'tauri/release/tauri-app.exe' } else { Join-Path $TargetRoot "slint-$v/release/slint-app.exe" }
        $work = Join-Path $TargetRoot "work-$v"
        New-Item -ItemType Directory -Force (Join-Path $work 'data') | Out-Null
        "##### $v"
        & $harness $exe $thumbs $work
        Start-Sleep -Seconds 3
    }
} finally {
    foreach ($n in $saved.Keys) { [Environment]::SetEnvironmentVariable($n, $saved[$n]) }
}
