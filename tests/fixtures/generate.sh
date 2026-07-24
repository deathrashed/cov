#!/bin/zsh
# Regenerates the committed audio/artwork fixtures. Requires: zsh, ffmpeg.
set -eu
cd "${0:A:h}"
SINE=(-f lavfi -i "sine=frequency=440:duration=1")
ffmpeg -y -loglevel error $SINE -c:a aac fixture.m4a
ffmpeg -y -loglevel error $SINE -c:a libmp3lame -b:a 64k fixture.mp3
ffmpeg -y -loglevel error $SINE -c:a flac fixture.flac
ffmpeg -y -loglevel error $SINE -ac 2 -c:a vorbis -strict -2 -b:a 64k fixture.ogg
ffmpeg -y -loglevel error $SINE -c:a libopus fixture.opus
ffmpeg -y -loglevel error $SINE -c:a pcm_s16le fixture.wav
ffmpeg -y -loglevel error $SINE -c:a pcm_s16be -f aiff fixture.aiff
ffmpeg -y -loglevel error $SINE -c:a wavpack fixture.wv
ffmpeg -y -loglevel error $SINE -c:a ape fixture.ape || print "SKIP: ape encoder unavailable (no ape encoder in FFmpeg; needs 'mac' binary)"
ffmpeg -y -loglevel error $SINE -c:a dsd_lsbf_planar -ar 2822400 fixture.dsf || print "SKIP: dsf encoder unavailable"
ffmpeg -y -loglevel error -f lavfi -i "color=c=0x01ACD7:size=256x256:duration=0.1" -frames:v 1 artwork.jpg
ffmpeg -y -loglevel error -f lavfi -i "color=c=0x1E1E1E:size=256x256:duration=0.1" -frames:v 1 artwork.png
