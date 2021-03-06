$SRC_DIR = $PWD.Path

# Generate release name
$CommitTime = (git show -s --format=%ct $APPVEYOR_REPO_COMMIT) | Out-String
$Date = (Get-Date).ToString("MM.dd")
$Env:RELEASE_NAME = "$($Date).$($CommitTime)".Trim()

$STAGE = [System.Guid]::NewGuid().ToString()
Set-Location $Env:Temp
New-Item -Type Directory -Name $STAGE
Set-Location $STAGE

New-Item -Type Directory -Name "$($Env:PACKAGE_NAME)-RELEASE"
New-Item -Type Directory -Name "$($Env:PACKAGE_NAME)-DEBUG"

$Bins = $Env:BIN_NAME -split ","
foreach ($Bin in $Bins) {
	Copy-Item "$SRC_DIR\target\$($Env:TARGET)\release\$Bin.exe" ".\$($Env:PACKAGE_NAME)-RELEASE\"
	Copy-Item "$SRC_DIR\target\$($Env:TARGET)\debug\$Bin.exe" ".\$($Env:PACKAGE_NAME)-DEBUG\"
}

7z a "$SRC_DIR\$($Env:PACKAGE_NAME)-$($Env:TARGET)-RELEASE.zip" "$($Env:PACKAGE_NAME)-RELEASE\"
7z a "$SRC_DIR\$($Env:PACKAGE_NAME)-$($Env:TARGET)-DEBUG.zip" "$($Env:PACKAGE_NAME)-DEBUG\"

Remove-Item *.* -Force
Set-Location ..
Remove-Item $STAGE -Force -Recurse
Set-Location $SRC_DIR

Push-AppveyorArtifact "$($Env:PACKAGE_NAME)-$($Env:TARGET)-RELEASE.zip"
Push-AppveyorArtifact "$($Env:PACKAGE_NAME)-$($Env:TARGET)-DEBUG.zip"

git tag $Env:RELEASE_NAME
