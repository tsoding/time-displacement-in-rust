#!/bin/sh

set -xe

mkdir -p input/
ffmpeg -y -i input.mp4 "input/%04d.png"
