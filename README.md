# psf

Reads psf (console) font files with optional support for automatic unzipping.

Decoding of the psf format was possible thanks to the [nafe](http://nafe.sourceforge.net/) tool.

## How to use

```rust
use psf::Font;

let the_font = Font::new("<path>");
if let Ok(font) = the_font {
    let c = font.get_char('X');
    if let Some(c) = c {
        println!("{:-<1$}", "", c.width() + 2);
        for h in 0..c.height() {
           print!("|");
           for w in 0..c.width() {
               let what = if c.get(w, h).unwrap() != 0 { "X" } else { " " };
               print!("{}", what);
           }
           println!("|");
       }
       println!("{:-<1$}", "", c.width() + 2);
    }
}
```

This is actually what method `Font::print_char()` is doing.
