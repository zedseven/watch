## Goal
Watch a file or directory, backing up every version

## Options
1. If only watching one file, can back up to `*.bak` files in-place
2. Can also back up to separate location/directory

### Additionally
- Can have options for naming scheme (`1`, `2`, `3`, etc, or date/time-based suffix)
- Can set interval for polling
- Hash-based or mod-date-based
- Create backup of file at start of watch (or not)
- Quiet (not silent) option
- Rename `beginning` option to `backup-starting-copy` or something
- Remove `Old` output and add timestamp
- Support file disappearance
