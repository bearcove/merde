mod cowstr;
mod lazyindexmap;

pub use cowstr::CowStr;
pub use lazyindexmap::LazyIndexMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'s> {
    Int(i64),
    Float(f64),
    String(CowStr<'s>),
    Null,
    Bool(bool),
    Array(Vec<Value<'s>>),
    Object(LazyIndexMap<CowStr<'s>, Value<'s>>),
}
