#!/bin/sh

set -xe

ffmpeg -y -i "output/%04d.png" output.mp4
