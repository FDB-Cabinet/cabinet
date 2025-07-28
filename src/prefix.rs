use fdb_wrapper::foundationdb::tuple::{TupleDepth, TuplePack, VersionstampOffset};
use std::io::Write;

#[derive(Debug, Copy, Clone)]
pub enum Prefix {
    Data = 0,
    Stats = 1,
}

impl TuplePack for Prefix {
    fn pack<W: Write>(
        &self,
        w: &mut W,
        tuple_depth: TupleDepth,
    ) -> std::io::Result<VersionstampOffset> {
        (*self as u64).pack(w, tuple_depth)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EntityType {
    Headcount = 0,
    Sizes = 1,
}

impl TuplePack for EntityType {
    fn pack<W: Write>(
        &self,
        w: &mut W,
        tuple_depth: TupleDepth,
    ) -> std::io::Result<VersionstampOffset> {
        (*self as u64).pack(w, tuple_depth)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum StatType {
    Value = 0,
    Sum = 1,
    Min = 2,
    Max = 3,
}

impl TuplePack for StatType {
    fn pack<W: Write>(
        &self,
        w: &mut W,
        tuple_depth: TupleDepth,
    ) -> std::io::Result<VersionstampOffset> {
        (*self as u64).pack(w, tuple_depth)
    }
}
