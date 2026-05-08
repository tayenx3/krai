# Integers and Floats

## Integers

Integers in Krai are an optional sign followed by a sequence of digits that represent a number.

```krai
10
-5
+255
123_456
```

There are 10 main integer types in Krai:

```krai
i8 i16 i32 i64 isz
u8 u16 u32 u64 usz
```

Integers can contain underscores anywhere after the first digit in the number part.

```krai
100_000_000
1_048_576
```

## Floats

Floats (floating-point) numbers in Krai are made from an optional sign, followed by an integer part, a dot and a decimal part.

```krai
0.5
-50.123
+78.12301
3.14159
```

There are 2 main float types in Krai:

```krai
f32 f64
```

After the first digit of the integer part, underscores can appear anywhere.

```krai
123_456.789
```

Even next to the dot:

```krai
123_.456
123._456
123_._456
```
