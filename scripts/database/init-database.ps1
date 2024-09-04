$ErrorActionPreference = "Stop"
Import-Module "$PSScriptRoot/PostgreSQL-Utilities.psm1"

Write-Header -Title "PostgreSQL database initialization"

$BaseSourcePath = Resolve-Path -Path (Join-Path -Path $PSScriptRoot -ChildPath "../..")
$BaseDataDirectory = Join-Path -Path $BaseSourcePath -ChildPath "data"
$DatabaseDataDirectory = Join-Path -Path $BaseDataDirectory -ChildPath "database"
$LogFilePath = Join-Path -Path $BaseDataDirectory -ChildPath "database.log"
Write-Log -Name "Initialization" -Content "Log file path: $LogFilePath"
Write-Log -Name "Initialization" -Content "Database directory: $DatabaseDataDirectory"

$PostgresPgCtlBinary = Get-PostgresBinary -BaseDirectory $PSScriptRoot -Binary "pg_ctl.exe"
$PostgresPsqlBinary = Get-PostgresBinary -BaseDirectory $PSScriptRoot -Binary "psql.exe"
Write-Log -Name "Initialization" -Content "Using pg_ctrl at $PostgresPgCtlBinary"
Write-Log -Name "Initialization" -Content "Using psql at $PostgresPsqlBinary"

If (Test-Path -Path $DatabaseDataDirectory -PathType Container) {
    Write-Log -Name "Initialization" -Content "There is already a database at $DatabaseDataDirectory." -Color Red
    Write-Log -Name "Initialization" -Content "Aborting - if you really wish to initialize, delete the data directory first." -Color Red
    exit 1
}

Write-Log -Name "Initialization" -Content "The superuser account will have the username `"postgres`"."
Write-Log -Name "Initialization" -Content "Note: remember the superuser password you're about to set!" -Color Yellow
Write-Log -Name "Initialization" -Content "Warning: will use `"--auth=trust`", don't use in production." -Color DarkRed
Write-Log -Name "Initialization" -Content "Initializing the PostgreSQL database (using pg_ctrl init)."
Invoke-Expression "$PostgresPgCtlBinary initdb -D `"$DatabaseDataDirectory`" -o `"--encoding=UTF8 --auth=trust --username=postgres --pwprompt`""

Write-Log -Name "Initialization" -Content "`"initdb`" finished, temporarily starting server to set up roles."
Invoke-Expression "$PostgresPgCtlBinary start -D `"$DatabaseDataDirectory`" -l `"$LogFilePath`""


Write-Log -Name "Initialization" -Content "Warning: using bad password, don't use in production." -Color DarkRed
Write-Log -Name "Initialization" -Content "Creating database stari_kolomoni..."
# Invoke-Expression "$PostgresPsqlBinary -h localhost -U postgres -c `"CREATE ROLE kolomon with PASSWORD 'kolomon' LOGIN`""
Invoke-Expression "$PostgresPsqlBinary -h localhost -U postgres -c `"CREATE DATABASE stari_kolomoni ENCODING UTF8`""
# Invoke-Expression "$PostgresPsqlBinary -h localhost -U postgres -c `"REVOKE CONNECT ON DATABASE kolomondb FROM PUBLIC`""
# Invoke-Expression "$PostgresPsqlBinary -h localhost -U postgres -c `"GRANT CONNECT ON DATABASE kolomondb TO postgres`""
# Invoke-Expression "$PostgresPsqlBinary -h localhost -U postgres -c `"GRANT CONNECT ON DATABASE kolomondb TO kolomon`""


Write-Log -Name "Initialization" -Content "Stopping PostgreSQL server."
Invoke-Expression "$PostgresPgCtlBinary stop -D `"$DatabaseDataDirectory`""

Write-Log -Name "Initialization" -Content "PostgresSQL database has been successfully initialized at `"$DatabaseDataDirectory`"." -Color Green
Write-Log -Name "Initialization" -Content "To start the database server, run `"run-database.ps1`"" -Color Green
