# Multi-Pattern Streams Detector

## Overview

Multi-Pattern Streams Detector is a file scanning and extraction command-line tool designed to detect and extract various stream types (formats) within a binary file. Uses the Aho-Corasick algorithm for efficient multi-pattern searching and processes files using memory-mapped I/O for performance.

## Usage

```
Multi-Pattern Streams Detector

Usage: mpsd [OPTIONS] <COMMAND>

Commands:
  scan     Scan the input file
  extract  Extract streams from the input file
  inject   Inject streams back to the input file from the given folder (supports large files on 32-bit systems)
  help     Print this message or the help of the given subcommand(s)

Options:
      --wav <DETECT_WAV>
          Enable WAV (RIFF WAVE PCM) detection [default: 1]
      --ogg <DETECT_OGG>
          Enable OGG detection [default: 1]
      --bmp <DETECT_BMP>
          Enable BMP (Windows BitMaP) detection [default: 1]
      --aac <DETECT_AAC>
          Enable AAC (ADTS) detection [default: 1]
      --mp3 <DETECT_MP3>
          Enable MP3 (MPEG-1/2 Audio) detection [default: 1]
      --mpeg-min-frames <MPEG_MIN_FRAMES>
          Minimum MPEG frames (0 = disabled) [default: 20]
      --mpeg-max-frames <MPEG_MAX_FRAMES>
          Maximum MPEG frames (0 = disabled) [default: 10000]
      --erase-sectors
          Replace the found sectors in the input file with zeros
          WARNING: backup the input file to prevent data loss
          Could be enabled only in "Extract" mode
      --rm
          Remove files after injection (only when injection was succeed)
          It won't delete a direction where files were located
  -s, --silent
          Do not print a log for each found/injected stream
  -h, --help
          Print help
```

## Large file support

Large files (> 4 GB) are supported only on 64-bit platforms (scan & extract modes) because of using memory-mapped I/O, which require addressing more memory space than 32-bit systems can provide. Injector automatically uses I/O if memory-map is not supported.

## Copyright and License

Copyright (c) 2024 Yurii Zhyvaha

This library is MIT licensed. See the
[LICENSE.md](https://github.com/phyxolog/mpsd/blob/master/LICENSE.md) for details.
