-- Keyboard Maestro / Script Editor: use Finder selection and embed the selected artwork.

tell application "Finder"
	set selectedItems to selection
	if selectedItems is {} then return "Select an audio file or album folder in Finder first."
	set selectedPath to POSIX path of (item 1 of selectedItems as alias)
end tell

set covHome to (POSIX path of (path to home folder)) & ".local/bin/cov"
return do shell script quoted form of covHome & " open --embed " & quoted form of selectedPath
