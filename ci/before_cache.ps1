Set-Location $APPVEYOR_BUILD_FOLDER
$Bins = $Env:BIN_NAME -split ","
foreach ($Bin in $Bins) {
	Remove-Item "target/$TARGET/debug/incremental/$($Bin)-*" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/debug/.fingerprint/$($Bin)-*" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/debug/build/$($Bin)-*" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/debug/deps/$($Bin)-*" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/debug/$($Bin).d" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/release/incremental/$($Bin)-*" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/release/.fingerprint/$($Bin)-*" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/release/build/$($Bin)-*" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/release/deps/$($Bin)-*" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/release/$($Bin).d" -Force -ErrorAction Ignore
}
cargo clean -p $Env:PACKAGE_NAME