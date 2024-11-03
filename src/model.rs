use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use indexmap::Equivalent;
use petgraph::{
    csr::DefaultIx,
    graph::{EdgeIndex, EdgeReference, NodeIndex},
    visit::EdgeRef,
    Directed, Graph,
};

pub type RoomId = NodeIndex<DefaultIx>;
pub type DoorId = EdgeIndex<DefaultIx>;

pub trait Door<R>: Sized + Clone + Eq + Hash {
    fn can_connect(&self, from: &R, to: &R) -> bool;
    fn is_directed(&self) -> bool;
    fn door_path_weight(&self) -> u16;
    fn get_all_doors() -> Vec<Self>;
}

#[derive(Debug, Clone)]
pub struct Key<R, D: Door<R>> {
    pub door: D,
    _phantom: PhantomData<R>,
}

impl<R, D: Door<R>> Key<R, D> {
    pub fn new(door: D) -> Self {
        Self {
            door,
            _phantom: PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DoorRef<R, D: Door<R>> {
    pub source_room_id: RoomId,
    pub door_id: DoorId,
    pub target_room_id: RoomId,
    pub door: D,
    _phantom: PhantomData<R>,
}

impl<R, D: Door<R>> DoorRef<R, D> {
    pub fn from_edge_reference(door_reference: &EdgeReference<D>) -> Self {
        DoorRef::<R, D> {
            source_room_id: door_reference.source().clone(),
            door_id: door_reference.id().clone(),
            target_room_id: door_reference.target().clone(),
            door: door_reference.weight().clone(),
            _phantom: PhantomData,
        }
    }
}

pub type TraversalGraph<R, D> = Graph<R, D, Directed, DefaultIx>;
