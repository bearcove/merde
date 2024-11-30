use crate::{DynSerializerExt, Event, IntoStatic, Map, MerdeError, Serializer, Value};
use insta::assert_debug_snapshot;

#[test]
fn test_serialize() {
    #[derive(Default, Debug)]
    struct ToySerializer {
        events: Vec<Event<'static>>,
    }

    impl Serializer for ToySerializer {
        fn write<'fut>(
            &'fut mut self,
            ev: Event<'fut>,
        ) -> impl std::future::Future<Output = Result<(), MerdeError<'static>>> + 'fut {
            self.events.push(ev.into_static());
            async { Ok(()) }
        }
    }

    let mut s = ToySerializer::default();
    let value: Value = Map::new().with("foo", Value::from(42)).into();
    s.serialize(&value).unwrap();

    assert_debug_snapshot!(s.events);
}
