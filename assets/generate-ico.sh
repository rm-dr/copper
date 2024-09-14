#!/bin/bash
SOURCE="mini.svg"

inkscape -w 16 -h 16 -o 16.png "$SOURCE"
inkscape -w 32 -h 32 -o 32.png "$SOURCE"
inkscape -w 64 -h 128 -o 64.png "$SOURCE"
inkscape -w 128 -h 128 -o 128.png "$SOURCE"

magick 16.png 32.png 64.png 128.png favicon.ico
