-- Keyboard Maestro / Script Editor: use the first selected Finder file or folder.

tell application "Finder"
	set selectedItems to selection
	if selectedItems is {} then return "Select an audio file or album folder in Finder first."
	set selectedPath to POSIX path of (item 1 of selectedItems as alias)
end tell

set covHome to (POSIX path of (path to home folder)) & ".local/bin/cov"
return do shell script quoted form of covHome & " open " & quoted form of selectedPath
