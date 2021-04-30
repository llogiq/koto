# tuple

Tuples in Koto are fixed contiguous arrays of values.

In contrast to Lists (which contains data that can modified),
once a tuple is created its data can't be modified.
Nested Lists and Maps in the Tuple can themselves be modified,
but the Tuple itself can be thought of as 'read-only'.

## Creating a Tuple

Tuples are created with comma-separated values:

```koto
x = "hello", -1, 99, [1, 2, 3]
x[2]
# 99
x[3]
# [1, 2, 3]
```

Parentheses are used when necessary for disambiguation:

```koto
x, y = (1, 2, 3), (4, 5, 6)
x[1], y[2]
# (2, 6)
```

# Reference

- [contains](#contains)
- [deep_copy](#deep_copy)
- [first](#first)
- [get](#get)
- [iter](#iter)
- [last](#last)
- [size](#size)
- [sort_copy](#sort_copy)
- [to_list](#to_list)

## contains

## deep_copy

## first

## get

## iter

## last

## size

## sort_copy

## to_list
