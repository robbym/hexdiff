use std::io::Read;

use ihex::{Reader, Record};

#[derive(Debug, Copy, Clone)]
pub struct IHex16Word {
    pub address: u32,
    pub value: u32,
}

pub struct IHex16File(pub Vec<IHex16Word>);

impl IHex16File {
    pub fn from_reader<R: Read>(read: &mut R) -> IHex16File {
        let mut hex_data = String::new();
        read.read_to_string(&mut hex_data).unwrap();

        let mut words: Vec<_> = Reader::new(&hex_data)
            .scan(0, |global_offset, record| match record {
                Ok(Record::Data { offset, value }) => Some(
                    value
                        .chunks(4)
                        .enumerate()
                        .map(|(idx, word)| {
                            let mut bytes = [0u8; 4];
                            bytes.copy_from_slice(word);

                            IHex16Word {
                                address: *global_offset as u32 + offset as u32 + (idx as u32 * 4),
                                value: u32::from_le_bytes(bytes),
                            }
                        })
                        .collect(),
                ),
                Ok(Record::ExtendedLinearAddress(offset)) => {
                    *global_offset = (offset as u32) << 16;
                    Some(vec![])
                }
                Ok(Record::EndOfFile) => None,
                _ => None,
            })
            .flatten()
            .collect();

        words.sort_by_key(|x| x.address);

        IHex16File(words)
    }
}
