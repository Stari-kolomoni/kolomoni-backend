$ErrorActionPreference = "Stop"
Import-Module "$PSScriptRoot/PostgreSQL-Utilities.psm1"

Write-Header -Title "PostgreSQL starter (non-interactive, non-blocking)"

$BaseSourcePath = Resolve-Path -Path (Join-Path -Path $PSScriptRoot -ChildPath "../..")
$BaseDataDirectory = Join-Path -Path $BaseSourcePath -ChildPath "data"
$DatabaseDataDirectory = Join-Path -Path $BaseDataDirectory -ChildPath "database"
$LogFilePath = Join-Path -Path $BaseDataDirectory -ChildPath "database.log"
Write-Log -Name "Initialization" -Content "Log file path: $LogFilePath"
Write-Log -Name "Initialization" -Content "Database directory: $DatabaseDataDirectory"

$PostgresPgCtlBinary = Get-PostgresBinary -BaseDirectory $PSScriptRoot -Binary "pg_ctl.exe"
Write-Log -Name "Runner" -Content "Using pg_ctrl at $PostgresPgCtlBinary"

If (-not (Test-Path -Path $DatabaseDataDirectory -PathType Container)) {
    Write-Log -Name "Runner" -Content "There is no database at $DatabaseDataDirectory." -Color Red
    Write-Log -Name "Runner" -Content "Aborting - run the initialization first (init-database.ps1)." -Color Red
    exit 1
}

Write-Log -Name "Runner" -Content "Starting PostgreSQL in the background (using pg_ctrl start)."
Invoke-Expression "$PostgresPgCtlBinary start -D `"$DatabaseDataDirectory`" -l `"$LogFilePath`""
Write-Log -Name "Runner" -Content "PostgreSQL server started. Use the manually-stop-database.ps1 script to stop it."
