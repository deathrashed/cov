-- Keyboard Maestro / Script Editor: save the selected COV artwork beside the album.

tell application "Swinsian"
	if not running then return "Swinsian is not running."
	set targetTrack to missing value
	try
		set selectedTracks to selection of front window
		if selectedTracks is not {} then set targetTrack to item 1 of selectedTracks
	end try
	if targetTrack is missing value then
		if player state is stopped then return "Select or play a track in Swinsian first."
		set targetTrack to current track
	end if
	set trackPath to path of targetTrack as text
end tell

set launcher to (do shell script "which cov-open || echo ~/.local/bin/cov-open")
return do shell script quoted form of launcher & " " & quoted form of trackPath
