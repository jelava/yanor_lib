use bevy::{platform::collections::HashMap, prelude::*};

// Basic idea: define each type of stat in an enum
// Stat block contains a list w/ exactly 1 of each enum variant
// TODO: strum crate seems like it will be very useful here

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StatId(pub u32);

// TODO: figure out if there's a way to use S::COUNT to use arrays/slices instead of vecs
#[derive(Component)]
pub struct StatBlock<T> {
    pub base_stats: HashMap<StatId, T>,
    // modifiers: EnumMap<M, Vec<StatModifier<T>>>,
    // adjusted_stats: EnumMap<S, T>,
    // _s: PhantomData<S>,
}

impl<T: Clone + Copy> StatBlock<T> {
    pub fn new(stats: &[(StatId, T)]) -> Self {
        let mut base_stats = HashMap::new();

        for (stat, value) in stats {
            base_stats.insert(*stat, *value);
        }

        Self { base_stats }
    }

    // fn get(&self, stat: S) -> T {
    //     self.adjusted_stats[stat]
    // }

    pub fn get_base(&self, stat: &StatId) -> Option<&T> {
        self.base_stats.get(stat)
    }

    pub fn set_base(&mut self, stat: StatId, value: T) {
        self.base_stats.insert(stat, value);

        // todo!("recalc adjusted stats");
    }

    // fn add_modifier(&mut self, stat: S, modifier: StatModifier<T>) {
    //     self.modifiers[stat].push(modifier);
    // }
}

impl<T: Default> Default for StatBlock<T> {
    fn default() -> Self {
        Self {
            base_stats: HashMap::default(),
            // modifiers: EnumMap::default(),
            // adjusted_stats: EnumMap::default(),
            // _s: PhantomData::default(),
        }
    }
}
