#[cfg(feature = "merde")]
use merde::CowStr;

#[cfg(not(feature = "merde"))]
pub type CowStr<'s> = std::borrow::Cow<'s, str>;

#[derive(Debug)]
pub struct Person<'s> {
    pub name: CowStr<'s>,
    pub age: u8, // sorry 256-year-olds
}

merde::derive! {
    impl (Deserialize, Serialize) for struct Person<'s> { name, age }
}
