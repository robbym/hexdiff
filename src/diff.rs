use std::iter::Fuse;
use std::vec;

use super::ihex16::{IHex16File, IHex16Word};

#[derive(Debug, Copy, Clone)]
pub enum IHex16Diff {
    Single {
        address: u32,
        value_1: u32,
        value_2: u32,
    },
    Range {
        start: u32,
        end: u32,
        value_1: u32,
        value_2: u32,
    },
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
        match self.compare() {
            Some((address, value_1, value_2)) => {
                if address > self.address {
                    let output = IHex16Diff::Range {
                        start: self.address,
                        end: address - 4,
                        value_1: 0xFFFFFF,
                        value_2: 0xFFFFFF,
                    };
                    self.address = address;
                    return Some(output);
                } else {
                    let mut next_address = address;
                    let mut next_value_1 = value_1;
                    let mut next_value_2 = value_2;

                    while next_value_1 == value_1 && next_value_2 == value_2 {
                        self.advance();
                        match self.compare() {
                            Some((na, nv1, nv2)) => {
                                if (na - 4) != next_address {
                                    break;
                                }

                                next_address += 4;
                                next_value_1 = nv1;
                                next_value_2 = nv2;
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    if address == next_address - 4 {
                        let output = IHex16Diff::Single {
                            address,
                            value_1,
                            value_2,
                        };
                        self.address += 4;
                        Some(output)
                    } else {
                        let output = IHex16Diff::Range {
                            start: address,
                            end: next_address - 4,
                            value_1,
                            value_2,
                        };
                        self.address = next_address;
                        Some(output)
                    }
                }
            }
            None => {
                return None;
            }
        }
    }
}
