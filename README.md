# watch
A simple Rust program for watching a file and making backups in-place when changes are detected.

## Usage
```
watch <watch-file> --interval <polling interval>
```

## Notes
Watch uses a 128-bit hashing algorithm to determine if files have changed, and pays no attention to file modification date.
