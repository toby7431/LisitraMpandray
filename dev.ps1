# Script de lancement du projet EgliseManager
# A utiliser depuis PowerShell ou en double-cliquant (Run with PowerShell)

# Trouver le Developer Command Prompt de VS pour configurer l'environnement MSVC
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"

if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath 2>$null
    if ($vsPath) {
        $vcvars = Join-Path $vsPath "VC\Auxiliary\Build\vcvars64.bat"
        if (Test-Path $vcvars) {
            Write-Host "Configuration de l'environnement MSVC..." -ForegroundColor Cyan
            # Charger l'environnement MSVC via cmd et lancer cargo dans la même session
            cmd /c "`"$vcvars`" && cd /d `"D:\lisitra_mpandray\EgliseManager`" && cargo tauri dev"
            exit
        }
    }
}

# Fallback : chercher link.exe directement
$linkPaths = @(
    "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC",
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
)

$linkExe = $null
foreach ($base in $linkPaths) {
    if (Test-Path $base) {
        $linkExe = Get-ChildItem $base -Recurse -Filter "link.exe" -ErrorAction SilentlyContinue |
                   Where-Object { $_.FullName -like "*HostX64\x64*" } |
                   Select-Object -First 1 -ExpandProperty FullName
        if ($linkExe) { break }
    }
}

if ($linkExe) {
    Write-Host "Linker MSVC trouve : $linkExe" -ForegroundColor Green
    $env:PATH = (Split-Path $linkExe) + ";" + $env:PATH
    Set-Location "D:\lisitra_mpandray\EgliseManager"
    cargo tauri dev
} else {
    Write-Host "ERREUR : MSVC link.exe introuvable." -ForegroundColor Red
    Write-Host "Visual Studio est peut-etre encore en cours d'installation." -ForegroundColor Yellow
    Write-Host "Attendez la fin de l'installation puis relancez ce script." -ForegroundColor Yellow
    Read-Host "Appuyez sur Entree pour quitter"
}
