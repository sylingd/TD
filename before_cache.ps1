$BIN_NAME = "twitchdl,m3u8"
$Bins = $BIN_NAME -split ","
foreach ($Bin in $Bins) {
	Remove-Item "target/$TARGET/debug/$($Bin).d" -Force -ErrorAction Ignore
	Remove-Item "target/$TARGET/release/$($Bin).d" -Force -ErrorAction Ignore
}
cargo clean -p twitchdl