use hibitset::BitSet;

use join::Join;
use storage::MaskedStorage;
use world::Component;
use Index;

/// A draining storage wrapper which has a `Join` implementation
/// that removes the components.
pub struct Drain<'a, T: Component> {
    /// The masked storage
    pub data: &'a mut MaskedStorage<T>,
}

impl<'a, T> Join for Drain<'a, T>
where
    T: Component,
{
    type Type = T;
    type Value = &'a mut MaskedStorage<T>;
    type Mask = BitSet;

    fn open(self) -> (Self::Mask, Self::Value) {
        let mask = self.data.mask.clone();

        (mask, self.data)
    }

    unsafe fn get(value: &mut Self::Value, id: Index) -> T {
        value.remove(id).expect("Tried to access same index twice")
    }
}

#[cfg(test)]
mod tests {
    use join::Join;
    use storage::{DenseVecStorage, NullStorage};
    use world::{Component, World};

    #[test]
    fn basic_drain() {
        #[derive(Debug, PartialEq)]
        struct Comp;

        impl Component for Comp {
            type Storage = DenseVecStorage<Self>;
        }

        let mut world = World::new();
        world.register::<Comp>();

        world.create_entity().build();
        let b = world.create_entity().with(Comp).build();
        let c = world.create_entity().with(Comp).build();
        world.create_entity().build();
        let e = world.create_entity().with(Comp).build();

        let mut comps = world.write::<Comp>();
        let entities = world.entities();

        {
            let mut iter = (comps.drain(), &*entities).join();

            assert_eq!(iter.next().unwrap(), (Comp, b));
            assert_eq!(iter.next().unwrap(), (Comp, c));
            assert_eq!(iter.next().unwrap(), (Comp, e));
        }

        assert_eq!((&comps).join().count(), 0);
    }

    #[test]
    fn selective_drain() {
        #[derive(Debug, Default, PartialEq)]
        struct Keep;
        impl Component for Keep { type Storage = NullStorage<Self>; }

        #[derive(Debug, Default, PartialEq)]
        struct Delete;
        impl Component for Delete { type Storage = NullStorage<Self>; }

        fn consume(_: Delete) {}

        let mut world = World::new();
        world.register::<Keep>();
        world.register::<Delete>();

        let a = world.create_entity().with(Delete).with(Keep).build();
        let b = world.create_entity().with(Delete).build();
        let c = world.create_entity().with(Keep).build();

        let mut del = world.write::<Delete>();
        let keep = world.write::<Keep>();
        let eid = world.entities();

        for (del, _, eid) in (del.drain(), !&keep, &*eid).join() {
            consume(del);
            assert!(eid != a);
            assert!(eid == b);
            assert!(eid != c);
        }

        assert_eq!((&keep).join().count(), 2);
        assert_eq!((&del).join().count(), 1);
    }
}
