# File

A map that wraps a file handle, returned from functions in `io`.

# Reference

- [path](#path)
- [read_to_string](#read_to_string)
- [seek](#seek)
- [write](#write)
- [write_line](#write_line)

## path

`|File| -> String`

Returns the file's path.

## read_to_string

`|File| -> String`

Reads the file's contents to a string.

### Errors

An error is thrown if the file doesn't contain valid UTF-8 data.

## seek

`|File, Number| -> ()`

Seeks within the file to the specified position in bytes.

## write

`|File, Value| -> ()`

Writes the formatted value as a string to the file.

## write_line

`|File, Value| -> ()`

Writes the formatted value as a string, with a newline, to the file.
