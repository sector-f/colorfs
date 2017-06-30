# colorfs

Usage: `colorfs MOUNTPOINT`

You can then read 1x1 PNG files from the mountpoint. Files are named in the form `rrggbb.png`,
where each color is a value between 00 and ff.

Example:

```sh
# Create the mountpoint
mkdir /tmp/colors

# Mount the filesystem
colorfs /tmp/colors &

# Make the desktop dark red
feh --bg-tile /colors/550000.png
```
