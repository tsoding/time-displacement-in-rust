#!/bin/sh

# Stolen from http://blog.pkh.me/p/21-high-quality-gif-with-ffmpeg.html

set -xe

palette="/tmp/palette.png"

filters="fps=30,scale=320:-1:flags=lanczos"

ffmpeg -v warning -i $1 -vf "$filters,palettegen" -y $palette
ffmpeg -v warning -i $1 -i $palette -lavfi "$filters [x]; [x][1:v] paletteuse" -y $2
