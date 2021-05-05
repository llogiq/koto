# map

Maps in Koto are associative containers of keys mapped to values.

The order in which items are added to the map will be preserved.

## Creating a map

There are two ways to directly create a map in Koto:
map blocks, and inline maps.

### Block map syntax

Maps can be created with indented blocks, where each line contains an entry of
the form `Key: Value`.

```koto
x =
  hello: -1
  goodbye: 99

x.hello
# -1
x.goodbye
# 99
```

Nested Maps can be defined with additional indentation:

```koto
x =
  hello:
    world: 99
    everybody: 123
    to:
      you: -1
x.hello.world
# 99
x.hello.to.you
# 123
```

### Inline map syntax

Maps can also be created with curly braces, with comma-separated entries.

```koto
x = {hello: -1, goodbye: "abc"}
x.hello
# -1
x.goodbye
# abc
```

If only the key is provided for an entry, then a value matching the name of the
key is looked for and is then copied into the entry.

```koto
hello = "abc"
goodbye = 99
x = {hello, goodbye, tschüss: 123}
x.goodbye
# 99
```

## Keys

When creating a Map directly, the keys are defined as strings.
To use non-string values as keys, [`map.insert`](#insert) can be used.

```koto
x = {}
x.insert 0, "Hello"
x.insert true, "World"
"{}, {}!".format x.get(0), x.get(true)
# Hello, World!
```

## Instance functions

When a Function is used as a value in a Map, and if it uses the keyword `self`
as its first argument, then the runtime will pass the instance of the map that
contains the function as the `self` argument.

```koto
x =
  # Initialize an empty list
  data: []
  # Takes a value and adds it to the list
  add_to_data: |self, n| self.data.push n
  # Returns the sum of the list
  sum: |self| self.data.sum()

x.add_to_data 2
x.add_to_data 20
x.sum()
# 22
```

## Operators

The `+` operator can be used to merge two maps together.

```koto
x = {hello: 123}
y = {goodbye: 99}
x + y
# {hello, goodbye}
```

### Meta Maps and overloaded operations

Maps can be used to create value types with custom behaviour.

Keys with `@` prefixes go into the map's 'meta map',
which is checked when the map is encountered in operations.

```koto
make_x = |n|
  data: n
  # Overloading the addition operator
  @+: |self, other|
    # a new instance is made with the result of adding the two values together
    make_x self.data + other.data
  # Overloading the subtraction operator
  @-: |self, other|
    make_x self.data - other.data

x1 = make_x 10
x2 = make_x 20

(x1 + x2).data
# 30
(x1 - x2).data
# -10
```

All binary operators can be overloaded following this pattern.

Additionally, the following meta functions can customize object behaviour:

- `@negate`
  - Overloads the unary negation operator:
    - `@negate: |self| make_x -self.data`
- `@index`
  - Overloads `[]` indexing:
    - `@index: |self, index| self.data + index`
- `@display`
  - Customizes how the map will be displayed when formatted as a string:
    - `@display: |self| "X: {}".format self.data`
- `@type`
  - Provides a String that's used when checking the map's type:
    - `@type: "X"`

# Reference

- [clear](#clear)
- [contains_key](#contains_key)
- [copy](#copy)
- [deep_copy](#deep_copy)
- [get](#get)
- [get_index](#get_index)
- [insert](#insert)
- [is_empty](#is_empty)
- [iter](#iter)
- [keys](#keys)
- [remove](#remove)
- [size](#size)
- [sort](#sort)
- [update](#update)
- [values](#values)

## clear

`|Map| -> ()`

Clears the map by removing all of its elements.

### Example

```koto
x = {x: -1, y: 42}
x.clear()
x
# {}
```

## contains_key

`|Map, Key| -> Bool`

Returns `true` if the map contains a value with the given key,
and `false` otherwise.

## copy

`|Map| -> Map`

Makes a unique copy of the map data.

Note that this only copies the first level of data, so nested containers
will share their data with their counterparts in the copy. To make a copy where
any nested containers are also unique, use [`map.deep_copy`](#deep_copy).

### Example

```koto
x = {foo: -1, bar: 99}
y = x
y.foo = 42
x.foo
# 42

z = x.copy()
z.bar = -1
x.bar # x.bar remains unmodified due to the
# 99
```

### See also

- [`map.deep_copy`](#deep_copy)

## deep_copy

`|Map| -> Map`

Makes a unique _deep_ copy of the map data.

This makes a unique copy of the map data, and then recursively makes deep copies
of any nested containers in the map.

If only the first level of data needs to be made unique, then use
[`map.copy`](#copy).

### Example

```koto
x = {foo: 42, bar: {baz: 99}}
y = m.deep_copy()
y.bar.baz = 123
x.bar.baz # a deep copy has been made, so x is unaffected by the change to y
# 99
```

### See also

- [`map.copy`](#copy)

## get

`|Map, Key| -> Value`

Returns the value corresponding to the given key, or `()` if the map doesn't
contain the key.

### Example

```koto
x = {hello: -1}
x.get "hello"
# -1

x.get "goodbye"
# ()

x.insert 99, "xyz"
x.get 99
# xyz
```

### See also

- [`map.get_index`](#get_index)

## get_index

`|Map, Number| -> Tuple`

Returns the entry at the given index as a key/value tuple, or `()` if the map
doesn't contain an entry at that index.

An error will be thrown if a negative index is provided.

### Example

```koto
x = {foo: -1, bar: -2}
x.get_index 1
# (bar, -2)

x.get_index 99
# ()
```

### See also

- [`map.get`](#get)

## insert

`|Map, Key| -> Value`

Inserts `()` into the map with the given key.

`|Map, Key, Value| -> Value`

Inserts a value into the map with the given key.

If the key already existed in the map, then the old value is returned.
If the key didn't already exist, then `()` is returned.

### Example

```koto
x = {hello: -1}
x.insert "hello", 99 # -1 already exists at `hello`, so it's returned here
# -1

x.hello # hello is now 99
# 99

x.insert "goodbye", 123 # No existing value at `goodbye`, so () is returned
# ()

x.goodbye
# 123
```

### See also

- [`map.remove`](#remove)
- [`map.update`](#update)

## is_empty

`|Map| -> Bool`

Returns `true` if the map contains no entries, otherwise `false`.

### Example

```koto
{}.is_empty()
# true

{hello: -1}.is_empty()
# false
```

### See also

- [`map.size`](#size)

## iter

`|Map| -> Iterator`

Returns an iterator that iterates over the map's entries.

Each key/value pair is provided in order as a tuple.

Maps are iterable, so it's not necessary to call `.iter()` to get access to
iterator operations, but it can be useful sometimes to make a standalone
iterator for manual iteration.

### Example

```koto
m =
  hello: -1
  goodbye: 99

x = m.iter();

x.next()
# ("hello", -1)

x.next()
# ("goodbye", 99)

x.next()
# ()
```

### See also

- [`map.keys`](#keys)
- [`map.values`](#values)

## keys

`|Map| -> Iterator`

Returns an iterator that iterates in order over the map's keys.

### Example

```koto
m =
  hello: -1
  goodbye: 99

x = m.keys()

x.next()
# "hello"

x.next()
# "goodbye"

x.next()
# ()
```

### See also

- [`map.iter`](#iter)
- [`map.values`](#values)

## remove

`|Map, Key| -> Value`

Removes the entry that matches the given key.

If the entry existed then its value is returned, otherwise `()` is returned.

### Example

```koto
x =
  hello: -1
  goodbye: 99

x.remove "hello"
# -1

x.remove "xyz"
# ()

x.remove "goodbye"
# 99

x.is_empty()
# true
```

### See also

- [`map.insert`](#insert)

## size

`|Map| -> Number`

Returns the number of entries contained in the map.

### Example

```koto
{}.size()
# 0

{"a": 0, "b": 1}.size()
# 2
```

### See also

- [`map.is_empty`](#is_empty)

## sort

`|Map| -> ()`

Sorts the map's entries by key.

`|Map, |Value, Value| -> Value| -> ()`

Sorts the map's entries, based on the output of calling a 'key' function for
each entry. The entry's key and value are passed into the function as separate
arguments.

The function result is cached, so it's only called once per entry.

### Example

```koto
x =
  hello: 123
  bye: -1
  tschüss: 99
x.sort() # Sorts the map by key
x
# {bye, hello, tschüss}

x.sort |_, value| value # Sort the map by value
x
# {bye, tschüss, hello}

x.sort |key, _| -key.size() # Sort the map by reversed key length
x
# {tschüss, hello, bye}
```

## update

`|Map, Key, |Value| -> Value| -> Value`

Updates the value associated with a given key by calling a function with either
the existing value, or `()` if it doesn't there isn't a matching entry.

The result of the function will either replace an existing value, or if no value
existed then an entry with the given key will be inserted into the map with the
function's result.

The function result is then returned from `update`.

`|Map, Key, Value, |Value| -> Value| -> Value`

This variant of `update` takes a default value that is provided to the
function if a matching entry doesn't exist.

### Example

```koto
x =
  hello: -1
  goodbye: 99

x.update "hello", |n| n * 2
# -2
x.hello
# -2

x.update "tschüss", 10, |n| n * 10
# 100
x.tschüss
# 100
```

### See also

- [`map.insert`](#insert)

## values

`|Map| -> Iterator`

Returns an iterator that iterates in order over the map's values.

### Example

```koto
m =
  hello: -1
  goodbye: 99

x = m.values()

x.next()
# -1

x.next()
# 99

x.next()
# ()
```

### See also

- [`map.iter`](#iter)
- [`map.keys`](#keys)
