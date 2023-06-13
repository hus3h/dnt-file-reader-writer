use std::{
    error::Error,
    fs::File,
    io::{Seek, SeekFrom},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug)]
pub enum DntDataType {
    String,
    Int32,
    Float32,
}

impl DntDataType {
    fn from_u8(value: u8) -> Self {
        match value {
            1 => DntDataType::String,
            2..=3 => DntDataType::Int32,
            4..=5 => DntDataType::Float32,
            other => panic!("Invalid column type value: {}", other),
        }
    }
}

#[derive(Debug)]
pub enum DntValue {
    String(String),
    Int32(i32),
    Float32(f32),
}

#[derive(Debug)]
pub struct DntColumn {
    pub text: String,
    data_type: DntDataType,
    raw_data_type: u8,
}

#[derive(Debug)]
pub struct DntRow {
    pub values: Vec<DntValue>,
}

pub struct DntTable {
    pub head: Vec<DntColumn>,
    pub body: Vec<DntRow>,
}

pub struct DntFileReader {
    file: File,
    data: DntTable,
}

impl DntFileReader {
    pub fn new(file: File) -> Self {
        Self {
            file,
            data: DntTable {
                head: vec![],
                body: vec![],
            },
        }
    }

    pub fn read(&mut self) -> Result<(), Box<dyn Error>> {
        self.seek(4);

        let mut data = DntTable {
            head: vec![DntColumn {
                text: String::from("id"),
                data_type: DntDataType::from_u8(3),
                raw_data_type: 3,
            }],
            body: vec![],
        };

        let columns_nb = self.read_u16() + 1;
        let rows_nb = self.read_u32();

        for _ in 1..columns_nb {
            let text = self.read_string();
            let raw_data_type = self.read_byte();
            let data_type = DntDataType::from_u8(raw_data_type);
            let column = DntColumn {
                text,
                data_type,
                raw_data_type,
            };
            data.head.push(column);
        }

        for _ in 0..rows_nb {
            let mut row = DntRow { values: vec![] };
            for column in &data.head {
                let value = match column.data_type {
                    DntDataType::String => DntValue::String(self.read_string()),
                    DntDataType::Int32 => DntValue::Int32(self.read_i32()),
                    DntDataType::Float32 => DntValue::Float32(self.read_f32()),
                };
                row.values.push(value);
            }
            data.body.push(row);
        }

        self.data = data;

        Ok(())
    }

    pub fn data(&mut self) -> &mut DntTable {
        &mut self.data
    }

    fn seek(&mut self, amount: u64) {
        self.file.seek(SeekFrom::Start(amount)).unwrap();
    }

    fn read_u16(&mut self) -> u16 {
        self.file.read_u16::<LittleEndian>().unwrap()
    }

    fn read_u32(&mut self) -> u32 {
        self.file.read_u32::<LittleEndian>().unwrap()
    }

    fn read_i32(&mut self) -> i32 {
        self.file.read_i32::<LittleEndian>().unwrap()
    }

    fn read_f32(&mut self) -> f32 {
        self.file.read_f32::<LittleEndian>().unwrap()
    }

    fn read_byte(&mut self) -> u8 {
        self.file.read_u8().unwrap()
    }

    fn read_string(&mut self) -> String {
        let length = self.read_u16() as usize;

        if length > 0 {
            let mut result = String::with_capacity(length);
            for index in 0..length {
                result.insert(index, self.read_byte() as char);
            }
            result
        } else {
            String::from("")
        }
    }
}

pub struct DntFileWriter {
    file: File,
}

impl DntFileWriter {
    pub fn new(file: File) -> Self {
        Self { file }
    }

    pub fn write(&mut self, table: &DntTable) -> Result<(), Box<dyn Error>> {
        self.write_byte(0);
        self.write_byte(0);
        self.write_byte(0);
        self.write_byte(0);

        self.write_u16(table.head.len() as u16 - 1);
        self.write_u32(table.body.len() as u32);

        for column_index in 1..table.head.len() {
            let column = table.head.get(column_index).unwrap();
            self.write_string(column.text.to_owned());
            self.write_byte(column.raw_data_type);
        }

        for row in &table.body {
            for value in &row.values {
                match value {
                    DntValue::String(value) => self.write_string(value.to_owned()),
                    DntValue::Int32(value) => self.write_i32(value.to_owned()),
                    DntValue::Float32(value) => self.write_f32(value.to_owned()),
                }
            }
        }

        let closing_text = String::from("THEND");

        self.write_byte(closing_text.len() as u8);
        self.write_string_bytes(closing_text);

        Ok(())
    }

    fn write_u16(&mut self, value: u16) {
        self.file.write_u16::<LittleEndian>(value).unwrap()
    }

    fn write_u32(&mut self, value: u32) {
        self.file.write_u32::<LittleEndian>(value).unwrap()
    }

    fn write_i32(&mut self, value: i32) {
        self.file.write_i32::<LittleEndian>(value).unwrap()
    }

    fn write_f32(&mut self, value: f32) {
        self.file.write_f32::<LittleEndian>(value).unwrap()
    }

    fn write_byte(&mut self, value: u8) {
        self.file.write_u8(value).unwrap()
    }

    fn write_string(&mut self, value: String) {
        self.write_u16(value.len() as u16);
        self.write_string_bytes(value);
    }

    fn write_string_bytes(&mut self, value: String) {
        for index in 0..value.len() {
            self.write_byte(value.chars().nth(index as usize).unwrap() as u8);
        }
    }
}
