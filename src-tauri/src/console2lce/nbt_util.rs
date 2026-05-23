use std::collections::HashMap;
use std::io::{Read, Write, Cursor};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression as FlateCompression;
use crate::console2lce::error::ConversionError;
#[derive(Debug, Clone)]
pub enum NbtTag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    ByteArray(Vec<u8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(Vec<NbtTag>, u8),
    Compound(HashMap<String, NbtTag>),
    End,
}

impl NbtTag {
    pub fn get_byte(&self) -> Option<i8> {
        if let NbtTag::Byte(v) = self { Some(*v) } else { None }
    }

    pub fn get_short(&self) -> Option<i16> {
        if let NbtTag::Short(v) = self { Some(*v) } else { None }
    }

    pub fn get_int(&self) -> Option<i32> {
        if let NbtTag::Int(v) = self { Some(*v) } else { None }
    }

    pub fn get_long(&self) -> Option<i64> {
        if let NbtTag::Long(v) = self { Some(*v) } else { None }
    }

    pub fn get_float(&self) -> Option<f32> {
        if let NbtTag::Float(v) = self { Some(*v) } else { None }
    }

    pub fn get_double(&self) -> Option<f64> {
        if let NbtTag::Double(v) = self { Some(*v) } else { None }
    }

    pub fn get_string(&self) -> Option<&str> {
        if let NbtTag::String(v) = self { Some(v.as_str()) } else { None }
    }

    pub fn get_byte_array(&self) -> Option<&[u8]> {
        if let NbtTag::ByteArray(v) = self { Some(v.as_slice()) } else { None }
    }

    pub fn get_int_array(&self) -> Option<&[i32]> {
        if let NbtTag::IntArray(v) = self { Some(v.as_slice()) } else { None }
    }

    pub fn get_long_array(&self) -> Option<&[i64]> {
        if let NbtTag::LongArray(v) = self { Some(v.as_slice()) } else { None }
    }

    pub fn get_list(&self) -> Option<(&[NbtTag], u8)> {
        if let NbtTag::List(tags, type_id) = self {
            Some((tags.as_slice(), *type_id))
        } else {
            None
        }
    }

    pub fn get_compound(&self) -> Option<&HashMap<String, NbtTag>> {
        if let NbtTag::Compound(map) = self { Some(map) } else { None }
    }

    pub fn get_compound_mut(&mut self) -> Option<&mut HashMap<String, NbtTag>> {
        if let NbtTag::Compound(map) = self { Some(map) } else { None }
    }

    pub fn as_compound(&self) -> Option<&HashMap<String, NbtTag>> {
        self.get_compound()
    }

    pub fn lookup(&self, path: &[&str]) -> Option<&NbtTag> {
        let mut current = self;
        for key in path {
            match current {
                NbtTag::Compound(map) => {
                    current = map.get(*key)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    pub fn lookup_mut(&mut self, path: &[&str]) -> Option<&mut NbtTag> {
        let mut current = self;
        for key in path {
            match current {
                NbtTag::Compound(map) => {
                    current = map.get_mut(*key)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }
}

fn read_string<R: Read>(reader: &mut R) -> Result<String, ConversionError> {
    let len = reader.read_u16::<BigEndian>()? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf).map_err(|e| ConversionError::NbtError(format!("Invalid UTF-8: {}", e)))?)
}

fn write_string<W: Write>(writer: &mut W, s: &str) -> Result<(), ConversionError> {
    let bytes = s.as_bytes();
    if bytes.len() > u16::MAX as usize {
        return Err(ConversionError::NbtError("String too long".to_string()));
    }
    writer.write_u16::<BigEndian>(bytes.len() as u16)?;
    writer.write_all(bytes)?;
    Ok(())
}

fn read_tag<R: Read>(reader: &mut R, tag_type: u8) -> Result<NbtTag, ConversionError> {
    match tag_type {
        0 => Ok(NbtTag::End),
        1 => Ok(NbtTag::Byte(reader.read_i8()?)),
        2 => Ok(NbtTag::Short(reader.read_i16::<BigEndian>()?)),
        3 => Ok(NbtTag::Int(reader.read_i32::<BigEndian>()?)),
        4 => Ok(NbtTag::Long(reader.read_i64::<BigEndian>()?)),
        5 => Ok(NbtTag::Float(reader.read_f32::<BigEndian>()?)),
        6 => Ok(NbtTag::Double(reader.read_f64::<BigEndian>()?)),
        7 => {
            let len = reader.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(ConversionError::NbtError("Negative byte array length".to_string()));
            }
            let mut buf = vec![0u8; len as usize];
            reader.read_exact(&mut buf)?;
            Ok(NbtTag::ByteArray(buf))
        }
        8 => Ok(NbtTag::String(read_string(reader)?)),
        9 => {
            let element_type = reader.read_u8()?;
            let len = reader.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(ConversionError::NbtError("Negative list length".to_string()));
            }
            let mut tags = Vec::with_capacity(len as usize);
            for _ in 0..len {
                tags.push(read_tag(reader, element_type)?);
            }
            Ok(NbtTag::List(tags, element_type))
        }
        10 => {
            let mut map = HashMap::new();
            loop {
                let item_type = reader.read_u8()?;
                if item_type == 0 { break; }
                let name = read_string(reader)?;
                let tag = read_tag(reader, item_type)?;
                map.insert(name, tag);
            }
            Ok(NbtTag::Compound(map))
        }
        11 => {
            let len = reader.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(ConversionError::NbtError("Negative int array length".to_string()));
            }
            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                buf.push(reader.read_i32::<BigEndian>()?);
            }
            Ok(NbtTag::IntArray(buf))
        }
        12 => {
            let len = reader.read_i32::<BigEndian>()?;
            if len < 0 {
                return Err(ConversionError::NbtError("Negative long array length".to_string()));
            }
            let mut buf = Vec::with_capacity(len as usize);
            for _ in 0..len {
                buf.push(reader.read_i64::<BigEndian>()?);
            }
            Ok(NbtTag::LongArray(buf))
        }
        _ => Err(ConversionError::NbtError(format!("Unknown tag type: {}", tag_type))),
    }
}

fn write_tag<W: Write>(writer: &mut W, tag: &NbtTag) -> Result<(), ConversionError> {
    match tag {
        NbtTag::End => writer.write_u8(0)?,
        NbtTag::Byte(v) => writer.write_i8(*v)?,
        NbtTag::Short(v) => writer.write_i16::<BigEndian>(*v)?,
        NbtTag::Int(v) => writer.write_i32::<BigEndian>(*v)?,
        NbtTag::Long(v) => writer.write_i64::<BigEndian>(*v)?,
        NbtTag::Float(v) => writer.write_f32::<BigEndian>(*v)?,
        NbtTag::Double(v) => writer.write_f64::<BigEndian>(*v)?,
        NbtTag::ByteArray(v) => {
            writer.write_i32::<BigEndian>(v.len() as i32)?;
            writer.write_all(v)?;
        }
        NbtTag::String(s) => write_string(writer, s)?,
        NbtTag::List(tags, _) => {
            let element_type = if tags.is_empty() { 0 } else { get_tag_type(&tags[0]) };
            writer.write_u8(element_type)?;
            writer.write_i32::<BigEndian>(tags.len() as i32)?;
            for tag in tags {
                write_tag(writer, tag)?;
            }
        }
        NbtTag::Compound(map) => {
            for (name, tag) in map {
                let tag_type = get_tag_type(tag);
                writer.write_u8(tag_type)?;
                write_string(writer, name)?;
                write_tag(writer, tag)?;
            }
            writer.write_u8(0)?;
        }
        NbtTag::IntArray(v) => {
            writer.write_i32::<BigEndian>(v.len() as i32)?;
            for val in v {
                writer.write_i32::<BigEndian>(*val)?;
            }
        }
        NbtTag::LongArray(v) => {
            writer.write_i32::<BigEndian>(v.len() as i32)?;
            for val in v {
                writer.write_i64::<BigEndian>(*val)?;
            }
        }
    }
    Ok(())
}

fn get_tag_type(tag: &NbtTag) -> u8 {
    match tag {
        NbtTag::End => 0,
        NbtTag::Byte(_) => 1,
        NbtTag::Short(_) => 2,
        NbtTag::Int(_) => 3,
        NbtTag::Long(_) => 4,
        NbtTag::Float(_) => 5,
        NbtTag::Double(_) => 6,
        NbtTag::ByteArray(_) => 7,
        NbtTag::String(_) => 8,
        NbtTag::List(_, _) => 9,
        NbtTag::Compound(_) => 10,
        NbtTag::IntArray(_) => 11,
        NbtTag::LongArray(_) => 12,
    }
}

pub fn read_nbt(data: &[u8]) -> Result<(String, NbtTag), ConversionError> {
    let mut reader = Cursor::new(data);
    let root_type = reader.read_u8()?;
    if root_type == 0 {
        return Ok((String::new(), NbtTag::End));
    }
    let name = read_string(&mut reader)?;
    let tag = read_tag(&mut reader, root_type)?;
    Ok((name, tag))
}

pub fn write_nbt(name: &str, tag: &NbtTag) -> Result<Vec<u8>, ConversionError> {
    let mut buf = Vec::new();
    let tag_type = get_tag_type(tag);
    buf.write_u8(tag_type)?;
    write_string(&mut buf, name)?;
    write_tag(&mut buf, tag)?;
    Ok(buf)
}

pub fn read_gzipped_nbt(data: &[u8]) -> Result<(String, NbtTag), ConversionError> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    read_nbt(&decompressed)
}

pub fn read_zlibbed_nbt(data: &[u8]) -> Result<(String, NbtTag), ConversionError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    read_nbt(&decompressed)
}

pub fn write_zlibbed_nbt(name: &str, tag: &NbtTag) -> Result<Vec<u8>, ConversionError> {
    let nbt_data = write_nbt(name, tag)?;
    let mut encoder = ZlibEncoder::new(Vec::new(), FlateCompression::default());
    encoder.write_all(&nbt_data)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}

pub fn write_nbt_le(tag: &NbtTag) -> Result<Vec<u8>, ConversionError> {
    let mut buf = Vec::new();
    let tag_type = get_tag_type(tag);
    buf.write_u8(tag_type)?;
    write_tag_le(&mut buf, tag)?;
    Ok(buf)
}

fn write_tag_le<W: Write>(writer: &mut W, tag: &NbtTag) -> Result<(), ConversionError> {
    match tag {
        NbtTag::End => writer.write_u8(0)?,
        NbtTag::Byte(v) => writer.write_i8(*v)?,
        NbtTag::Short(v) => writer.write_i16::<LittleEndian>(*v)?,
        NbtTag::Int(v) => writer.write_i32::<LittleEndian>(*v)?,
        NbtTag::Long(v) => writer.write_i64::<LittleEndian>(*v)?,
        NbtTag::Float(v) => writer.write_f32::<LittleEndian>(*v)?,
        NbtTag::Double(v) => writer.write_f64::<LittleEndian>(*v)?,
        NbtTag::ByteArray(v) => {
            writer.write_i32::<LittleEndian>(v.len() as i32)?;
            writer.write_all(v)?;
        }
        NbtTag::String(s) => {
            let bytes = s.as_bytes();
            writer.write_u16::<LittleEndian>(bytes.len() as u16)?;
            writer.write_all(bytes)?;
        }
        NbtTag::List(tags, _) => {
            let element_type = if tags.is_empty() { 0 } else { get_tag_type(&tags[0]) };
            writer.write_u8(element_type)?;
            writer.write_i32::<LittleEndian>(tags.len() as i32)?;
            for t in tags {
                write_tag_le(writer, t)?;
            }
        }
        NbtTag::Compound(map) => {
            for (name, t) in map {
                let tt = get_tag_type(t);
                writer.write_u8(tt)?;
                let bytes = name.as_bytes();
                writer.write_u16::<LittleEndian>(bytes.len() as u16)?;
                writer.write_all(bytes)?;
                write_tag_le(writer, t)?;
            }
            writer.write_u8(0)?;
        }
        NbtTag::IntArray(v) => {
            writer.write_i32::<LittleEndian>(v.len() as i32)?;
            for val in v {
                writer.write_i32::<LittleEndian>(*val)?;
            }
        }
        NbtTag::LongArray(v) => {
            writer.write_i32::<LittleEndian>(v.len() as i32)?;
            for val in v {
                writer.write_i64::<LittleEndian>(*val)?;
            }
        }
    }
    Ok(())
}

pub fn read_mcr_chunk(data: &[u8], _compressed_size: u32) -> Result<Vec<u8>, ConversionError> {
    if data.len() < 5 {
        return Err(ConversionError::InvalidFormat("Chunk data too short".to_string()));
    }
    let compression_scheme = data[4];
    let payload = &data[5..];
    match compression_scheme {
        1 => {
            let mut decoder = GzDecoder::new(payload);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        }
        2 => {
            let mut decoder = ZlibDecoder::new(payload);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        }
        3 => {
            Ok(payload.to_vec())
        }
        _ => Err(ConversionError::DecompressionFailed(
            format!("Unknown compression scheme: {}", compression_scheme)
        )),
    }
}

#[derive(Debug, Clone)]
pub struct NbtHelper;
impl NbtHelper {
    pub fn get_byte_array(tag: &HashMap<String, NbtTag>, name: &str) -> Option<Vec<u8>> {
        tag.get(name).and_then(|t| t.get_byte_array().map(|v| v.to_vec()))
    }

    pub fn get_byte_array_or_default(tag: &HashMap<String, NbtTag>, name: &str, default_size: usize) -> Vec<u8> {
        Self::get_byte_array(tag, name).map(|v| {
            if v.len() >= default_size {
                v[..default_size].to_vec()
            } else {
                let mut padded = vec![0u8; default_size];
                padded[..v.len()].copy_from_slice(&v);
                padded
            }
        }).unwrap_or_else(|| vec![0u8; default_size])
    }

    pub fn get_int(tag: &HashMap<String, NbtTag>, name: &str) -> Option<i32> {
        tag.get(name).and_then(|t| t.get_int())
    }

    pub fn get_long(tag: &HashMap<String, NbtTag>, name: &str) -> Option<i64> {
        tag.get(name).and_then(|t| t.get_long())
    }

    pub fn get_short(tag: &HashMap<String, NbtTag>, name: &str) -> Option<i16> {
        tag.get(name).and_then(|t| t.get_short())
    }

    pub fn get_byte(tag: &HashMap<String, NbtTag>, name: &str) -> Option<i8> {
        tag.get(name).and_then(|t| t.get_byte())
    }

    pub fn get_string(tag: &HashMap<String, NbtTag>, name: &str) -> Option<String> {
        tag.get(name).and_then(|t| t.get_string().map(|s| s.to_string()))
    }

    pub fn get_list(tag: &HashMap<String, NbtTag>, name: &str) -> Option<Vec<NbtTag>> {
        tag.get(name).and_then(|t| {
            if let NbtTag::List(tags, _) = t {
                Some(tags.clone())
            } else {
                None
            }
        })
    }

    pub fn get_compound(tag: &HashMap<String, NbtTag>, name: &str) -> Option<HashMap<String, NbtTag>> {
        tag.get(name).and_then(|t| {
            if let NbtTag::Compound(map) = t {
                Some(map.clone())
            } else {
                None
            }
        })
    }
}
