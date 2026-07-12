param(
  [string]$Repository = "talal-ai/handoff-now",
  [string]$Version = "latest"
)
$ErrorActionPreference = "Stop"
$arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()) {
  "Arm64" { "aarch64-pc-windows-msvc" }
  "X64" { "x86_64-pc-windows-msvc" }
  default { throw "Unsupported Windows architecture" }
}
$release = if ($Version -eq "latest") { "latest/download" } else { "download/$Version" }
$name = "handoff-now-$arch.exe"
$base = "https://github.com/$Repository/releases/$release"
$temp = Join-Path ([IO.Path]::GetTempPath()) ("handoff-now-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $temp | Out-Null
try {
  Invoke-WebRequest "$base/$name" -OutFile (Join-Path $temp $name)
  Invoke-WebRequest "$base/SHA256SUMS" -OutFile (Join-Path $temp "SHA256SUMS")
  $expectedLine = Select-String -Path (Join-Path $temp "SHA256SUMS") -Pattern ([regex]::Escape($name) + '$') | Select-Object -First 1
  if (-not $expectedLine) { throw "Checksum entry not found for $name" }
  $expected = ($expectedLine.Line -split '\s+')[0].ToLowerInvariant()
  $actual = (Get-FileHash -Algorithm SHA256 (Join-Path $temp $name)).Hash.ToLowerInvariant()
  if ($expected -ne $actual) { throw "Checksum mismatch" }
  & (Join-Path $temp $name) setup
  & "$HOME\.claude\handoff-now\bin\handoff-now.exe" doctor
} finally {
  Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
