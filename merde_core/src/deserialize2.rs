pub enum Primitive {
    Int(i64),
    Float(f64),
    Str(String),
    Null,
    Bool(bool),
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
}

pub enum StepIn {
    // the parser encountered a primitive
    Primitive(Primitive),

    // a nested deserializer just finished
    Pop(DeserializeCx<()>),
}

pub enum StepOut {
    // the primitive passed in was consumed
    Consumed,

    // the primitive passed in was not consumed, we
    // want another deserializer to handle it, and
    Push(DeserializeCx<()>),

    //
    Done,
}

pub struct DeserializeVtable<State, Output> {
    pub step: fn(state: &mut State, step: StepIn) -> StepOut,
    pub consume: fn(state: &mut State) -> Output,
}

pub struct DeserializeCx<State> {
    pub data: *const State,
    pub vtbl: DeserializeVtable<State>,
}

pub trait DeserializeTrait {
    type Data;
    fn deserialize() -> DeserializeCx<Self::Data>;
}

pub fn dostuff() {
    let stack: Vec<DeserializeCx<u8>> = vec![];

    // FIXME: can we use async for this? if parsing was async, then
    // await points could return until the child was deserialized?
}
