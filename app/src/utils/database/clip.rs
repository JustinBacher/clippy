use std::{borrow::Borrow, iter::Iterator, ops::RangeBounds};

use impl_trait_for_tuples::impl_for_tuples;
use redb::{AccessGuard, Key, Range, ReadableTable, ReadableTableMetadata, Table, Value};

use crate::prelude::{Result, W};

struct Entry {
    k: i64,
    v: Vec<u8>,
}
