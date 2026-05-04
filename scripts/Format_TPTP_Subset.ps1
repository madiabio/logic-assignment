param(
    [Parameter(Mandatory=$true)]
    [string]$InputFile,

    [Parameter(Mandatory=$true)]
    [string]$Difficulty,

    [string]$CompareFile = $null
)

$OutputFile = "${Difficulty}_problems.txt"

$CommandUsed = "tptp2t -q3 -pp Form FOF Status Theorem Formulae 0 50 Atoms 0 150 -Equality -Arithmetic"
$Description = "Unbounded rating, between 0 and 50 formulae, 0 and 150 atoms. No equality, no arithmetic."
$SortDescription = "Sorted by number of atoms ascending."

$ColumnHeader = "% Problem            Frm SZS   Rtng SPC                     FmlCls  UnitF  TypeF  Atoms EquAts  Conns  FOOLs Ariths  Types Symbls  Preds  Arity  Funcs  Arity   Vars      ^      !      ?     !>    {.}    {#}"

function Get-ProblemRows {
    param([string]$Path)

    Get-Content $Path |
        Where-Object {
            $_.Trim() -ne "" -and
            -not $_.TrimStart().StartsWith("%")
        }
}

function Get-ProblemName {
    param([string]$Line)

    ($Line -split '\s+')[0]
}

function Get-Atoms {
    param([string]$Line)

    $cols = $Line -split '\s+'

    # Column positions:
    # 0 Problem
    # 1 Frm
    # 2 SZS
    # 3 Rtng
    # 4 SPC
    # 5 FmlCls
    # 6 UnitF
    # 7 TypeF
    # 8 Atoms
    [int]$cols[8]
}

$rows = Get-ProblemRows $InputFile

$sortedRows =
    $rows |
    Sort-Object @{ Expression = { Get-Atoms $_ }; Ascending = $true },
                @{ Expression = { Get-ProblemName $_ }; Ascending = $true }

$output = @(
    "% $CommandUsed > $InputFile"
    "% $Description"
    "% $SortDescription"
    $ColumnHeader
) + $sortedRows

$output | Set-Content $OutputFile

Write-Host "Wrote $OutputFile"

if ($CompareFile) {
    $a = Get-ProblemRows $InputFile | ForEach-Object { Get-ProblemName $_ }
    $b = Get-ProblemRows $CompareFile | ForEach-Object { Get-ProblemName $_ }

    $overlap = $a | Where-Object { $b -contains $_ } | Sort-Object -Unique

    if ($overlap.Count -eq 0) {
        Write-Host "No overlapping problems with ${CompareFile}"
    } else {
        Write-Host "Overlapping problems with ${CompareFile}:"
        $overlap
    }
}
