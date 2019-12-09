use std::iter::Fuse;
use std::vec;

use serde_derive::Serialize;

use super::ihex16::{IHex16File, IHex16Word};

#[derive(Copy, Clone, Serialize)]
#[serde(tag = "type")]
pub enum IHex16Diff {
    #[serde(rename(serialize = "single"))]
    Single {
        address: u32,
        value_1: u32,
        value_2: u32,
    },

    #[serde(rename(serialize = "range"))]
    Range {
        start: u32,
        end: u32,
        value_1: u32,
        value_2: u32,
    },
}

impl IHex16Diff {
    fn range(start: u32, end: u32, value_1: u32, value_2: u32) -> IHex16Diff {
        IHex16Diff::Range {
            start,
            end,
            value_1,
            value_2,
        }
    }

    fn single(address: u32, value_1: u32, value_2: u32) -> IHex16Diff {
        IHex16Diff::Single {
            address,
            value_1,
            value_2,
        }
    }

    pub fn is_same(&self) -> bool {
        match self {
            IHex16Diff::Single { address: _, value_1, value_2 } => value_1 == value_2,
            IHex16Diff::Range { start: _, end: _, value_1, value_2 } => value_1 == value_2
        }
    }

    pub fn is_diff(&self) -> bool {
        match self {
            IHex16Diff::Single { address: _, value_1, value_2 } => value_1 != value_2,
            IHex16Diff::Range { start: _, end: _, value_1, value_2 } => value_1 != value_2
        }
    }
}

pub struct IHex16DiffEngine {
    hex_1: Fuse<vec::IntoIter<IHex16Word>>,
    hex_2: Fuse<vec::IntoIter<IHex16Word>>,
    address: u32,
    curr_1: Option<IHex16Word>,
    curr_2: Option<IHex16Word>,
}

impl IHex16DiffEngine {
    pub fn diff(hex_1: IHex16File, hex_2: IHex16File) -> IHex16DiffEngine {
        let mut hex_1 = hex_1.0.into_iter().fuse();
        let mut hex_2 = hex_2.0.into_iter().fuse();
        let curr_1 = hex_1.next();
        let curr_2 = hex_2.next();

        IHex16DiffEngine {
            hex_1,
            hex_2,
            address: 0,
            curr_1,
            curr_2,
        }
    }

    fn compare(&self) -> Option<(u32, u32, u32)> {
        match (self.curr_1, self.curr_2) {
            (None, None) => None,
            (Some(l), None) => Some((l.address, l.value, 0xFFFFFF)),
            (None, Some(r)) => Some((r.address, 0xFFFFFF, r.value)),
            (Some(l), Some(r)) => {
                if l.address == r.address {
                    Some((l.address, l.value, r.value))
                } else if l.address < r.address {
                    Some((l.address, l.value, 0xFFFFFF))
                } else {
                    Some((r.address, 0xFFFFFF, r.value))
                }
            }
        }
    }

    fn advance(&mut self) {
        match (self.curr_1, self.curr_2) {
            (None, None) => {
                return;
            }
            (Some(_), None) => {
                self.curr_1 = self.hex_1.next();
            }
            (None, Some(_)) => {
                self.curr_2 = self.hex_2.next();
            }
            (Some(l), Some(r)) => {
                if l.address <= r.address {
                    self.curr_1 = self.hex_1.next();
                }

                if r.address <= l.address {
                    self.curr_2 = self.hex_2.next();
                }
            }
        }
    }
}

impl Iterator for IHex16DiffEngine {
    type Item = IHex16Diff;

    fn next(&mut self) -> Option<Self::Item> {
        let (address, value_1, value_2) = self.compare()?;

        let mut next_address = address;
        let mut next_value_1 = value_1;
        let mut next_value_2 = value_2;

        if address > self.address {
            next_value_1 = 0xFFFFFF;
            next_value_2 = 0xFFFFFF;
        }

        while next_value_1 == value_1 && next_value_2 == value_2 {
            self.advance();

            if let Some((na, nv1, nv2)) = self.compare() {
                if (na - 4) != next_address {
                    if next_value_1 == 0xFFFFFF && next_value_2 == 0xFFFFFF {
                        next_address = na - 4;
                    } else {
                        break;
                    }
                }

                next_address += 4;
                next_value_1 = nv1;
                next_value_2 = nv2;
            } else {
                break;
            }
        }

        if address == next_address - 4 {
            let output = IHex16Diff::single(address / 2, value_1, value_2);
            self.address += 4;
            Some(output)
        } else {
            let output = IHex16Diff::range(
                address / 2,
                (next_address - 4) / 2,
                value_1,
                value_2,
            );
            self.address = next_address;
            Some(output)
        }
    }
}
