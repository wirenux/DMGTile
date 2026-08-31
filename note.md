# how work the .bin gamwboy tile encoding

If we take this list of byte (on a single line) : `Black, Gray, Light Gray, White``
The binary (before the gameboy thing; just the palette) would be : `11, 10, 01, 00`

The final file will be :

```hex
A0 C0 00 00 00 ...
```

The "translation" of this hexadecimal file is : 
```binary
10100000 11000000 00000000 00000000 ...
```

*We will say that the first byte is A and the 2nd is B*

To get the Black pixel we need to get the 1st bit of B and add the 1st bit of A to create `11` which is the Black pixel

So if we want to get the Gray pixel we do the same thing but with the 2nd bit since this is the 2nd pixel of the row.
We get the 2nd bit of B (which is `1`) then add it to the 2nd bit of A (`0`) to get `10` (Grey pixel)


So each row is 2 byte
