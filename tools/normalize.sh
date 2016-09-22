#!/usr/bin/sh

if [ $# -eq 0]
then
	echo "a file is required" ^&1
else
	FILE="$1"
	mv "$FILE" "$FILE.tmp"
	ffmpeg -i "$FILE.tmp" -vn -acodec copy "$FILE.ogg"
	sox --norm=-3 "$FILE.ogg" "$FILE.norm.ogg"
	ffmpeg -i "$FILE.tmp" -i "$FILE.norm.ogg" -map 0:0 -map 0:1 -map 1 -c copy -metadata:s:a:1 title="normalized" "$FILE" && rm "$1.tmp" "$1.norm.ogg" "$1.ogg"
fi
