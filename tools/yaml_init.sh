#!/bin/sh
SCRIPTDIR=$(dirname "$0")
for file in "$@"
do
	sh $SCRIPTDIR/ffmpeg_video_length.sh "$file" | python3 $SCRIPTDIR/video_to_yaml.py "$file"
done
