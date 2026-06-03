param(
    [Parameter(Mandatory = $true)]
    [string] $OutputDir
)

$ErrorActionPreference = "Stop"

$defaultToolkitUrl = "https://archive.org/download/microsoft-visual-c-toolkit-2003/VCToolkitSetup.exe"
$defaultPlatformSdkUrl = "https://download.microsoft.com/download/7/5/e/75ec7f04-4c8c-4f38-b582-966e76602643/5.2.3790.1830.15.PlatformSDK_Svr2003SP1_rtm.img"

$toolchainZipUrl = $env:VC71_TOOLCHAIN_URL
$toolkitUrl = if ($env:VCTOOLKIT2003_URL) { $env:VCTOOLKIT2003_URL } else { $defaultToolkitUrl }
$platformSdkUrl = if ($env:PLATFORM_SDK_URL) { $env:PLATFORM_SDK_URL } else { $defaultPlatformSdkUrl }

function Invoke-Download {
    param(
        [Parameter(Mandatory = $true)] [string] $Uri,
        [Parameter(Mandatory = $true)] [string] $OutFile
    )

    Write-Host "Downloading $Uri"
    Invoke-WebRequest -Uri $Uri -OutFile $OutFile
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

if (Test-Path $OutputDir) {
    Remove-Item -Path $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

if ($toolchainZipUrl) {
    $zipPath = Join-Path $env:RUNNER_TEMP "vc71-toolchain.zip"
    Invoke-Download -Uri $toolchainZipUrl -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath $OutputDir -Force
} else {
    $toolkitExe = Join-Path $env:RUNNER_TEMP "VCToolkitSetup.exe"
    $toolkitExtract = Join-Path $env:RUNNER_TEMP "vctoolkit-extract"
    Invoke-Download -Uri $toolkitUrl -OutFile $toolkitExe
    Expand-AnyArchive -Archive $toolkitExe -Destination $toolkitExtract

    $cl = Find-FirstFile -Root $toolkitExtract -Name "cl.exe"
    if (-not $cl) {
        $toolkitInstall = Join-Path $env:RUNNER_TEMP "vctoolkit-install"
        New-Item -ItemType Directory -Force -Path $toolkitInstall | Out-Null

        Write-Host "7-Zip did not expose cl.exe; running Toolkit installer silently"
        $installerArgs = @(
            "/s",
            "/v`"/qn INSTALLDIR=`"$toolkitInstall`"`""
        )
        $process = Start-Process -FilePath $toolkitExe -ArgumentList $installerArgs -Wait -PassThru
        if ($process.ExitCode -ne 0) {
            throw "Visual C++ Toolkit installer failed with exit code $($process.ExitCode)"
        }

        $cl = Find-FirstFile -Root $toolkitInstall -Name "cl.exe"
        if ($cl) {
            $toolkitExtract = $toolkitInstall
        }
    }

    if (-not $cl) {
        throw "Could not find cl.exe after unpacking Visual C++ Toolkit 2003"
    }

    $vcRoot = Split-Path -Parent (Split-Path -Parent $cl.FullName)
    Copy-Tree -Source $vcRoot -Destination (Join-Path $OutputDir "VC7")

    $sdkImage = Join-Path $env:RUNNER_TEMP "PlatformSDK.img"
    Invoke-Download -Uri $platformSdkUrl -OutFile $sdkImage

    $sdkExtract = Join-Path $env:RUNNER_TEMP "platform-sdk-extract"
    Expand-AnyArchive -Archive $sdkImage -Destination $sdkExtract

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
