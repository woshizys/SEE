pub trait ItemSize {
    fn size_of(&self) -> usize;
}

impl ItemSize for u8 { fn size_of(&self) -> usize { 1 } }
impl ItemSize for u16 { fn size_of(&self) -> usize { 2 } }
impl ItemSize for u32 { fn size_of(&self) -> usize { 4 } }
impl ItemSize for u64 { fn size_of(&self) -> usize { 8 } }
impl ItemSize for u128 { fn size_of(&self) -> usize { 16 } }
impl ItemSize for usize { fn size_of(&self) -> usize { 8 } }
impl ItemSize for i8 { fn size_of(&self) -> usize { 1 } }
impl ItemSize for i16 { fn size_of(&self) -> usize { 2 } }
impl ItemSize for i32 { fn size_of(&self) -> usize { 4 } }
impl ItemSize for i64 { fn size_of(&self) -> usize { 8 } }
impl ItemSize for i128 { fn size_of(&self) -> usize { 16 } }
impl ItemSize for isize { fn size_of(&self) -> usize { 8 } }
impl ItemSize for f32 { fn size_of(&self) -> usize { 4 } }
impl ItemSize for f64 { fn size_of(&self) -> usize { 8 } }
impl ItemSize for char { fn size_of(&self) -> usize { 1 } }
impl ItemSize for bool { fn size_of(&self) -> usize { 1 } }
impl ItemSize for String { fn size_of(&self) -> usize { self.len() } }
impl ItemSize for &str { fn size_of(&self) -> usize { self.len() } }
impl ItemSize for [&u8] { fn size_of(&self) -> usize { self.iter().len() } }
impl<T> ItemSize for Vec<T>
where
    T: ItemSize,
{
    fn size_of(&self) -> usize { self.len() * size_of::<T>() }
}

impl ItemSize for () { fn size_of(&self) -> usize { 0 } }