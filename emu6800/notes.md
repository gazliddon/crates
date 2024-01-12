# Toy Language for emulating 8 bit processors

# Toy Language for emulating 8 bit processors
* Types
    * struct
    * fn
    * byte
    * bord
    * flag
    * bus
        * direction
        * width

* Misc
    * Pass by value
    * No loops
    * Control flow

literals
    num
    bool
    fn

# Declarations
<identifier> :: <type> 
<identifier> :: <type> <expression>

x :: u8 0

y :: [u8] 

z :: String "hello"

hello :: (u8 u8 u8) -> u8 = (a b c) {
    a + b + c
}

hello :: (u8 u8 u8) -> u8 = (a b c) -> d { 
    if a == 0 
        a + b
    else 
        b + c
}

## Declaration and Assignment

## Declaration
```
    <identifier> :: <type>
```

## Declaration and Assignment
```
    <identifier> :: <type> = <expression>
    <identifier> := <expression>
```

## Primitive Types

### Functions
```
    (a:u8, b:u8, c:u8) -> u8 {
    }
```



# Assignment
<identifier> = expression

# Function

* Operations
    * All wrapping
    * + - * / ! & ^ shl shr

Flags :: struct {
    Z :: Flag,
    N :: Flag,
    V :: Flag,
    C :: Flag,
    H :: Flag,
    I :: Flag,
}

Regs :: struct {
    A :: byte,
    B :: byte,
    X :: byte,
    SP :: byte,
    PC :: byte,
    SR :: Flags,
}

ldaa_im :: (r : Regs, b: Bus) = {
    b.set(PC)
    PC+
    a := load_flags_8(b,r.SR);
    r.A = a;
}

ext :: (b: Bus, addr: word) {
    b.set(addr);
    addr =  b.read_16();
    b.set(addr);
}

ldaa_ext:: (r : Regs, b: Bus) = {
    ext(b,r.pc)
    r.A := load_flags_8(b,r.SR);
}

ldab_ext:: (r : Regs, b: Bus) = {
    ext(b,r.pc)
    r.B := load_flags_8(b,r.SR);
}

load_flags_8 :: (b: Bus,f : Flags) -> byte = {
    a := b.load_mem_8();
    f.C = false;
    f.N = a[7];
    f.Z = a;
    a
}

LD_A : addr
    A = [addr]
    C = 0
    N = A:7
    Z = A

DIR addr = [PC]:b
    LD_A addr
    PC+

IMM LD_A PC
    PC+

EXT addr = [PC]:w
    LD_A addr
    PC++


