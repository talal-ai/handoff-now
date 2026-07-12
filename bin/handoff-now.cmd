@echo off
setlocal
set "HANDOFF_NOW=%USERPROFILE%\.claude\handoff-now\bin\handoff-now.exe"
if not exist "%HANDOFF_NOW%" (
  echo handoff-now is not initialized. Run the handoff-now setup command. 1>&2
  exit /b 127
)
"%HANDOFF_NOW%" %*
