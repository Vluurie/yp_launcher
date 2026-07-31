$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

cargo build --manifest-path rust/Cargo.toml --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

flutter build windows --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$bundle = 'build\windows\x64\runner\Release'
Copy-Item 'rust\target\release\yp_3d_inspector.dll' $bundle
