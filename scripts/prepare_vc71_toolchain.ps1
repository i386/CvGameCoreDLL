param(
    [Parameter(Mandatory = $true)]
    [string] $OutputDir,

    [string] $DownloadCacheDir = $env:VC71_DOWNLOAD_CACHE
)

$ErrorActionPreference = "Stop"

$defaultToolkitUrl = "https://archive.org/download/microsoft-visual-c-toolkit-2003/VCToolkitSetup.exe"
$defaultVs2003IsoUrl = "https://archive.org/download/vsnet2003/MSDN%20Visual%20Studio%20NET%202003%20-%20Enterprise%20Architect%20%28Disc%201%29%28Disc%202082%29%28May%202003%29%28X09-51498%29.ISO"
$defaultPlatformSdkUrl = "https://download.microsoft.com/download/7/5/e/75ec7f04-4c8c-4f38-b582-966e76602643/5.2.3790.1830.15.PlatformSDK_Svr2003SP1_rtm.img"

$toolchainZipUrl = $env:VC71_TOOLCHAIN_URL
$vs2003IsoUrl = if ($env:VS2003_ISO_URL) { $env:VS2003_ISO_URL } else { $defaultVs2003IsoUrl }
$toolkitUrl = if ($env:VCTOOLKIT2003_URL) { $env:VCTOOLKIT2003_URL } else { $defaultToolkitUrl }
$platformSdkUrl = if ($env:PLATFORM_SDK_URL) { $env:PLATFORM_SDK_URL } else { $defaultPlatformSdkUrl }

function Invoke-Download {
    param(
        [Parameter(Mandatory = $true)] [string] $Uri,
        [Parameter(Mandatory = $true)] [string] $OutFile
    )

    if ($DownloadCacheDir) {
        New-Item -ItemType Directory -Force -Path $DownloadCacheDir | Out-Null
        $extension = [System.IO.Path]::GetExtension(([Uri] $Uri).AbsolutePath)
        if (-not $extension) {
            $extension = ".download"
        }

        $sha256 = [System.Security.Cryptography.SHA256]::Create()
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($Uri)
        $hash = [BitConverter]::ToString($sha256.ComputeHash($bytes)).Replace("-", "").ToLowerInvariant()
        $cacheFile = Join-Path $DownloadCacheDir ($hash + $extension)

        if (Test-Path $cacheFile) {
            Write-Host "Using cached download for $Uri"
            Copy-Item -Path $cacheFile -Destination $OutFile -Force
            return
        }
    } else {
        $cacheFile = $null
    }

    Write-Host "Downloading $Uri"
    Invoke-WebRequest -Uri $Uri -OutFile $OutFile

    if ($cacheFile) {
        Copy-Item -Path $OutFile -Destination $cacheFile -Force
    }
}

function Expand-AnyArchive {
    param(
        [Parameter(Mandatory = $true)] [string] $Archive,
        [Parameter(Mandatory = $true)] [string] $Destination
    )

    New-Item -ItemType Directory -Force -Path $Destination | Out-Null

    if ($Archive -match "\.zip$") {
        Expand-Archive -Path $Archive -DestinationPath $Destination -Force
        return
    }

    $sevenZip = "${env:ProgramFiles}\7-Zip\7z.exe"
    if (-not (Test-Path $sevenZip)) {
        $sevenZip = "${env:ProgramFiles(x86)}\7-Zip\7z.exe"
    }
    if (-not (Test-Path $sevenZip)) {
        throw "7-Zip is required to unpack $Archive"
    }

    & $sevenZip x $Archive "-o$Destination" -y
    if ($LASTEXITCODE -ne 0) {
        throw "7-Zip failed to unpack $Archive"
    }
}

function Expand-NestedArchives {
    param(
        [Parameter(Mandatory = $true)] [string] $Root
    )

    $sevenZip = "${env:ProgramFiles}\7-Zip\7z.exe"
    if (-not (Test-Path $sevenZip)) {
        $sevenZip = "${env:ProgramFiles(x86)}\7-Zip\7z.exe"
    }
    if (-not (Test-Path $sevenZip)) {
        throw "7-Zip is required to unpack nested archives"
    }

    for ($pass = 1; $pass -le 4; $pass++) {
        $archives = Get-ChildItem -Path $Root -Recurse -File -ErrorAction SilentlyContinue |
            Where-Object { $_.Extension -match '^\.(cab|msi|zip|exe)$' -and $_.FullName -notmatch '\\expanded-' }

        foreach ($archive in $archives) {
            $destination = Join-Path $archive.DirectoryName ("expanded-" + $archive.BaseName)
            if (Test-Path $destination) {
                continue
            }

            New-Item -ItemType Directory -Force -Path $destination | Out-Null
            Write-Host "Expanding nested archive $($archive.FullName)"
            & $sevenZip x $archive.FullName "-o$destination" -y
            if ($LASTEXITCODE -ne 0) {
                Write-Host "Skipping nested archive that 7-Zip could not unpack: $($archive.FullName)"
            }
        }
    }
}

function Copy-Tree {
    param(
        [Parameter(Mandatory = $true)] [string] $Source,
        [Parameter(Mandatory = $true)] [string] $Destination
    )

    New-Item -ItemType Directory -Force -Path $Destination | Out-Null
    Copy-Item -Path (Join-Path $Source "*") -Destination $Destination -Recurse -Force
}

function Find-FirstFile {
    param(
        [Parameter(Mandatory = $true)] [string] $Root,
        [Parameter(Mandatory = $true)] [string] $Name
    )

    Get-ChildItem -Path $Root -Filter $Name -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
}

function Find-VcCompiler {
    param(
        [Parameter(Mandatory = $true)] [string] $Root
    )

    $matches = Get-ChildItem -Path $Root -Filter "cl.exe" -Recurse -ErrorAction SilentlyContinue
    $preferred = $matches |
        Where-Object { $_.FullName -match '\\Vc7\\bin\\cl\.exe$' } |
        Select-Object -First 1
    if ($preferred) {
        return $preferred
    }

    $preferred = $matches |
        Where-Object { $_.FullName -match 'Visual C\+\+ Toolkit 2003\\bin\\cl\.exe$' } |
        Select-Object -First 1
    if ($preferred) {
        return $preferred
    }

    $matches | Select-Object -First 1
}

function Invoke-ProcessWithTimeout {
    param(
        [Parameter(Mandatory = $true)] [string] $FilePath,
        [Parameter(Mandatory = $true)] [string] $Arguments,
        [int] $TimeoutSeconds = 90
    )

    $process = Start-Process -FilePath $FilePath -ArgumentList $Arguments -PassThru
    $timeoutMilliseconds = $TimeoutSeconds * 1000
    if (-not $process.WaitForExit($timeoutMilliseconds)) {
        Write-Host "Process timed out after $TimeoutSeconds seconds; killing process tree for PID $($process.Id)"
        & taskkill.exe /PID $process.Id /T /F | ForEach-Object { Write-Host $_ }
        $process.WaitForExit()
        return $null
    }

    return $process.ExitCode
}

if (Test-Path $OutputDir) {
    Remove-Item -Path $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

if ($toolchainZipUrl) {
    $zipPath = Join-Path $env:RUNNER_TEMP "vc71-toolchain.zip"
    Invoke-Download -Uri $toolchainZipUrl -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath $OutputDir -Force
} else {
    $vsIso = Join-Path $env:RUNNER_TEMP "VS2003.iso"
    $vsExtract = Join-Path $env:RUNNER_TEMP "vs2003-extract"
    Invoke-Download -Uri $vs2003IsoUrl -OutFile $vsIso
    Expand-AnyArchive -Archive $vsIso -Destination $vsExtract

    $cl = Find-VcCompiler -Root $vsExtract
    $vcSourceRoot = $null
    if ($cl) {
        $vcSourceRoot = Split-Path -Parent (Split-Path -Parent $cl.FullName)
    }

    if (-not $cl) {
        $toolkitExe = Join-Path $env:RUNNER_TEMP "VCToolkitSetup.exe"
        $toolkitExtract = Join-Path $env:RUNNER_TEMP "vctoolkit-extract"
        Invoke-Download -Uri $toolkitUrl -OutFile $toolkitExe
        Expand-AnyArchive -Archive $toolkitExe -Destination $toolkitExtract
        Expand-NestedArchives -Root $toolkitExtract

        Write-Host "Visual C++ Toolkit extracted files:"
        Get-ChildItem -Path $toolkitExtract -Recurse -File -ErrorAction SilentlyContinue |
            Select-Object -First 80 |
            ForEach-Object { Write-Host $_.FullName }

        $cl = Find-VcCompiler -Root $toolkitExtract
        if (-not $cl) {
            $toolkitInstall = Join-Path $env:RUNNER_TEMP "vctoolkit-install"
            New-Item -ItemType Directory -Force -Path $toolkitInstall | Out-Null

            Write-Host "7-Zip did not expose cl.exe; trying InstallShield/MSI extraction modes"
            $installAttempts = @(
                "/s /v`"/qn INSTALLDIR=`"$toolkitInstall`" /L*v `"$env:RUNNER_TEMP\vctoolkit-install.log`"`"",
                "/a /s /v`"/qn TARGETDIR=`"$toolkitInstall`" /L*v `"$env:RUNNER_TEMP\vctoolkit-admin.log`"`"",
                "/s /a /s /v`"/qn TARGETDIR=`"$toolkitInstall`" /L*v `"$env:RUNNER_TEMP\vctoolkit-admin2.log`"`"",
                "/v`"/qn INSTALLDIR=`"$toolkitInstall`" /L*v `"$env:RUNNER_TEMP\vctoolkit-install2.log`"`""
            )

            foreach ($installerArgs in $installAttempts) {
                Write-Host "Running $toolkitExe $installerArgs"
                $env:__COMPAT_LAYER = "WINXPSP3"
                $exitCode = Invoke-ProcessWithTimeout -FilePath $toolkitExe -Arguments $installerArgs -TimeoutSeconds 90
                Write-Host "Installer exit code: $exitCode"

                $cl = Find-VcCompiler -Root $toolkitInstall
                if ($cl) {
                    $toolkitExtract = $toolkitInstall
                    break
                }
            }
        }

        if ($cl) {
            $vcSourceRoot = Split-Path -Parent (Split-Path -Parent $cl.FullName)
        }
    }

    if (-not $cl) {
        Get-ChildItem -Path $env:RUNNER_TEMP -Filter "vctoolkit-*.log" -ErrorAction SilentlyContinue |
            ForEach-Object {
                Write-Host "---- $($_.FullName) ----"
                Get-Content -Path $_.FullName -Tail 80 -ErrorAction SilentlyContinue
            }
        throw "Could not find cl.exe after unpacking Visual Studio 2003 media or Visual C++ Toolkit 2003"
    }

    Copy-Tree -Source $vcSourceRoot -Destination (Join-Path $OutputDir "VC7")

    $sdkImage = Join-Path $env:RUNNER_TEMP "PlatformSDK.img"
    Invoke-Download -Uri $platformSdkUrl -OutFile $sdkImage

    $sdkExtract = Join-Path $env:RUNNER_TEMP "platform-sdk-extract"
    Expand-AnyArchive -Archive $sdkImage -Destination $sdkExtract
    Expand-NestedArchives -Root $sdkExtract

    $windowsHeader = Find-FirstFile -Root $sdkExtract -Name "windows.h"
    if (-not $windowsHeader) {
        throw "Could not find windows.h after unpacking Platform SDK image"
    }

    $includeDir = Split-Path -Parent $windowsHeader.FullName
    $sdkRoot = Split-Path -Parent $includeDir
    Copy-Tree -Source $sdkRoot -Destination (Join-Path $OutputDir "PlatformSDK")
}

$foundCl = Find-FirstFile -Root $OutputDir -Name "cl.exe"
$foundWindows = Find-FirstFile -Root $OutputDir -Name "windows.h"
$foundWinmm = Find-FirstFile -Root $OutputDir -Name "winmm.lib"

if (-not $foundCl) { throw "Prepared toolchain is missing cl.exe" }
if (-not $foundWindows) { throw "Prepared toolchain is missing windows.h" }
if (-not $foundWinmm) { throw "Prepared toolchain is missing winmm.lib" }

Write-Host "Prepared VC7.1 toolchain at $OutputDir"
Write-Host "cl.exe: $($foundCl.FullName)"
Write-Host "windows.h: $($foundWindows.FullName)"
Write-Host "winmm.lib: $($foundWinmm.FullName)"
