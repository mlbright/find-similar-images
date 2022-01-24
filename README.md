# find-similar-images

## Overview

This program takes a single directory as a command line argument and compares all the `.jpeg` and `.png` files in the directory to determine their similarity using an algorithm approximating human vision.

It can be slow for large directories, even if the [library that computes (dis)similarity][dssim] is fast, and there is a reasonable degree of parallelism thanks to the amazing [rayon][rayon]. This is because it compares each image to every other image _of the same size_.

I used it to find multiple gigabytes of identical pictures in my MacOS Photos app directory.

```bash
find-similar-images "/Users/mlbright/Pictures/Photos Library.photoslibrary/originals" | tee different-yet-similar.txt
```

A similarity score of `0.000000` means the files are identical.
Very small values mean very similar images.
If the similarity score is not `0.000000`, visually inspect the images to get a sense of the numbers before deleting any files.

**This program will not attempt to delete any files.**

## Build

```bash
cargo build --release
```

## Missing features

The program should be able to detect identical files quickly and avoid a relatively expensive similarity score calculation.
However, this isn't implemented.
It assumes the directory has had duplicate files removed and that only different, _similar_ files remain.

One can find duplicate files to remove from a directory via something like:

```bash
find "$some_directory" -type f -exec md5sum {} \; | sort | awk 'visited[$1]++' | awk '{$1=""; print}' | tee duplicates-to-delete.txt
```

[rayon]: https://docs.rs/rayon/latest/rayon/
[dssim]: https://github.com/kornelski/dssim
