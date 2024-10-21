use crate::{Event, IntoStatic, Map, Serializer, Value};
use insta::assert_debug_snapshot;

#[test]
fn test_serialize() {
    #[derive(Default, Debug)]
    struct ToySerializer {
        events: Vec<Event<'static>>,
    }

    impl Serializer for ToySerializer {
        type Error = ();

        fn write(
            &mut self,
            ev: Event<'_>,
        ) -> impl std::future::Future<Output = Result<(), Self::Error>> {
            self.events.push(ev.into_static());
            async { Ok(()) }
        }
    }

    let mut s = ToySerializer::default();
    let value: Value = Map::new().with("foo", Value::from(42)).into();
    s.serialize_sync(&value).unwrap();

    assert_debug_snapshot!(s.events);
}
