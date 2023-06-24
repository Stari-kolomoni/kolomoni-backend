$ErrorActionPreference = "Stop"
Import-Module "$PSScriptRoot/PostgreSQL-Utilities.psm1"

Write-Header -Title "PostgreSQL runner (non-interactive)"

$BaseDatabasePath = Resolve-Path -Path (Join-Path -Path $PSScriptRoot -ChildPath "..")
$LogFilePath = Join-Path -Path $BaseDatabasePath -ChildPath "database.log"
$DatabaseDataDirectory = Join-Path -Path $BaseDatabasePath -ChildPath "data"
Write-Log -Name "Runner" -Content "Log file path: $LogFilePath"
Write-Log -Name "Runner" -Content "Data path: $DatabaseDataDirectory"

$PostgresPgCtlBinary = Get-PostgresBinary -BaseDirectory $PSScriptRoot -Binary "pg_ctl.exe"
Write-Log -Name "Runner" -Content "Using pg_ctrl at $PostgresPgCtlBinary"

If (-not (Test-Path -Path $DatabaseDataDirectory -PathType Container)) {
    Write-Log -Name "Runner" -Content "There is no database at $DatabaseDataDirectory." -Color Red
    Write-Log -Name "Runner" -Content "Aborting - run the initialization first (init-database.ps1)." -Color Red
    exit 1
}

Write-Log -Name "Runner" -Content "Starting PostgreSQL in the background (using pg_ctrl start)."
Invoke-Expression "$PostgresPgCtlBinary start -D `"$DatabaseDataDirectory`" -l `"$LogFilePath`""
Write-Log -Name "Runner" -Content "PostgreSQL server started, press Ctrl+C to gracefully stop the server."

try {
    while($true) {
        Start-Sleep -Milliseconds 10
    }
} finally {
    Write-Log -Name "Runner" -Content "Shutting down PostgreSQL server." -Color Yellow
    Invoke-Expression "$PostgresPgCtlBinary stop -D `"$DatabaseDataDirectory`" -m smart"
    Write-Log -Name "Runner" -Content "PostgreSQL server stopped." -Color Green
    exit 0
}