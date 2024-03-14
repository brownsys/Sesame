### What it doeS
Denies non-library implementations of AlohomoraType. 

### Why is this bad?
Developers must derive impls of AlohomoraType to ensure integrity of policy protection.

### Example
```rust
// impl AlohomoraType for BadStruct { ... }
 ```
 Use instead:
```rust
// #[derive(AlohomoraType)]
// #[out_type(name = "GoodStructOut", to_derive = [Debug])]
// pub struct GoodStruct { ... }    /// ```
