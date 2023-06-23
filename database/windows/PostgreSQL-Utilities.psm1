function Write-Header {
    <#
     # This function will print a colourful script header,
     # including a title and a description.
     #>
    [CmdletBinding()]
    Param (
        [Parameter(Mandatory=$true)]
        [String] $Title,

        [Parameter(Mandatory=$false)]
        [String] $Description = ""
    )

    Write-Host -ForegroundColor DarkYellow "=="
    Write-Host -ForegroundColor DarkYellow "== $Title"

    If (-not ($Description -eq "")) {
        Write-Host -ForegroundColor DarkYellow "== $Description"
    }

    Write-Host -ForegroundColor DarkYellow "=="
    Write-Host
}

function Write-Log {
    <#
     # This function will color a log entry with the format "[log title] log content".
     # Square brackets will be blue, the log name cyan and the content white.
     #>
    [CmdletBinding()]
    Param (
        [Parameter(Mandatory=$true)]
        [String] $Name,

        [Parameter(Mandatory=$true)]
        [String] $Content,

        [Parameter(Mandatory=$false)]
        [ConsoleColor] $Color = [ConsoleColor]::White
    )

    Write-Host -ForegroundColor Blue "[" -NoNewline
    Write-Host -ForegroundColor Cyan "$Name" -NoNewline
    Write-Host -ForegroundColor Blue "] " -NoNewline
    Write-Host -ForegroundColor $Color "$Content"
}


function Get-PostgresBinary {
    <#
     # This function will attempt to find the postgres binaries.
     #
     # It will first look in the base directory for a folder named `pgsql/bin` (the portable binaries folder).
     # If it does not find it there or the folder does not exist, it will attempt to find one in your `PATH`.
     # If that fails, it will throw an error.
     #>
    [CmdletBinding()]
    Param (
        [Parameter(Mandatory=$true)]
        [String] $BaseDirectory,

        [Parameter(Mandatory=$true)]
        [String] $Binary
    )

    $LocalBinary = Join-Path -Path $BaseDirectory -ChildPath "pgsql/bin/$Binary"

    If (-not (Test-Path $LocalBinary -PathType Leaf)) {
        # Check in `PATH` first.
        If (Get-Command $Binary -ErrorAction SilentlyContinue) {
            $PgCtl = Get-Command $Binary
            return $PgCtl.Source
        } Else {
            throw "Unable to find $Binary in $BaseDirectory or PATH."
        }
    } Else {
        return $LocalBinary
    }
}