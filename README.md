# Multi-Pattern Streams Detector

## Overview

The Multi-Pattern Streams Detector is a file scanning and extraction command-line tool designed to detect and extract various stream types (formats) within a binary file. Uses the Aho-Corasick algorithm for efficient multi-pattern searching and processes files using memory-mapped I/O for performance.

## Usage

```sh
Usage: mpsd [OPTIONS] <COMMAND>

Commands:
  scan     Scan input file
  extract  Extract streams from input file
  help     Print this message or the help of the given subcommand(s)

Options:
      --wav <DETECT_WAV>  Enable WAV (RIFF WAVE PCM) detection [default: 1]
      --ogg <DETECT_OGG>  Enable OGG detection [default: 1]
      --bmp <DETECT_BMP>  Enable BMP (Windows BitMaP) detection [default: 1]
      --aac <DETECT_AAC>  Enable AAC (ADTS) detection [default: 1]
      --mp3 <DETECT_MP3>  Enable MP3 (MPEG-1/2 Audio) detection [default: 1]
  -h, --help              Print help
```

## Copyright and License

Copyright (c) 2024 Yurii Zhyvaha

This library is MIT licensed. See the
[LICENSE.md](https://github.com/phyxolog/mpsd/blob/master/LICENSE.md) for details.
