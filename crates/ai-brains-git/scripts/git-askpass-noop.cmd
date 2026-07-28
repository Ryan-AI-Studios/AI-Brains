@echo off
REM No-op GIT_ASKPASS helper for non-interactive automation.
REM Git invokes this as: askpass.exe "Password for ..."
REM Exit 0 immediately with empty stdout so credential prompts fail closed
REM without hanging on a TTY/GUI prompt.
exit /b 0
