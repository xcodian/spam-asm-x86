# spam-asm x86_64

A Rust procedural macro that adds 5-20 lines of random assembly between every line of code you've written. This helps you create polymorphic executables that are really good at passing exact-comparisons, and are obfuscated to a degree.

Using this may help with evading 

Say you have this code:
```rust
fn main() {
    println!("something");
    println!("something else");
    println!("something elseeeee");
    for i in 0..100 {
        println!("example #{}", i);
    }

    let mut x = 1;
    println!("more statements");
    
    x += 1;
    println!("x = {}", x);
}
```

You can slap the attribute `#[spam_asm]` on it like so:
```rust
use spam_asm::spam_asm;

#[spam_asm]
fn main() {
    /* ... */
}
```

And the result of this is:
```rust
fn main() {
    unsafe { 
        asm!( /* do-nothing asm */ ) 
    }
    println!("something");
    unsafe { 
        asm!( /* do-nothing asm */ ) 
    }
    println!("something else");
    unsafe { 
        asm!( /* do-nothing asm */ ) 
    }
    println!("something elseeeee");
    unsafe { 
        asm!( /* do-nothing asm */ ) 
    }
    for i in 0..100 {
        unsafe { 
            asm!( /* do-nothing asm */ ) 
        }
        println!("example #{}", i);
    }
    unsafe { 
        asm!( /* do-nothing asm */ ) 
    }
    let mut x = 1;
    unsafe { 
        asm!( /* do-nothing asm */ ) 
    }
    println!("more statements");
    
    unsafe { 
        asm!( /* do-nothing asm */ ) 
    }
    x += 1;

    unsafe { 
        asm!( /* do-nothing asm */ ) 
    }
    println!("x = {}", x);
}
```

The "do-nothing asm" can be one of of the following operations:

**For all registers:**
- `mov reg, reg`
- `add reg, 0x00000000`
- `sub reg, 0x00000000`
- `xor reg, 0x00000000`
- `and reg, 0xffffffff`
- `and reg, reg`
- `or reg, 0x00000000`
- `or reg, reg`
- `xchg reg, reg`

**+ for 32-bit registers:**

- `lea reg,  [reg]`
- `ror reg,  0x20`
- `rol reg,  0x20`

**+ for 8 bit registers:**
- `ror reg,  0x08`
- `rol reg,  0x08`

**+ finally for MM registers**
- `pand reg, reg`
- `por reg, reg`

Pretty much all of these operations are performed on the same
register, so they effectively do nothing.

The actual macro inserts between 5 and 20 of the above instructions,
also selecting a random register to do it on.