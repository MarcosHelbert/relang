#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Str(String),
}

pub fn serialize_value(val: &Value) -> Vec<u8> {
    match val {
        Value::Int(i) => {
            let mut out = vec![0x01]; // Type: Int
            out.extend(&i.to_le_bytes());
            out
        }
        Value::Bool(b) => {
            vec![0x02, if *b { 1 } else { 0 }] // Type: Bool
        }
        Value::Str(s) => {
            let bytes = s.as_bytes();
            let len = bytes.len();
            assert!(len <= (u16::MAX as usize), "String too long");

            let mut out = vec![0x03]; // Type: String
            out.extend(&(len as u16).to_le_bytes());
            out.extend(bytes);
            out
        }
    }
}

pub fn deserialize_value(data: &[u8]) -> Option<(Value, usize)> {
    let tag = data.get(0)?;
    match tag {
        0x01 => {
            if data.len() < 5 {
                return None;
            }
            let int_bytes: [u8; 4] = data[1..5].try_into().ok()?;
            Some((Value::Int(i32::from_le_bytes(int_bytes)), 5))
        }
        0x02 => {
            if data.len() < 2 {
                return None;
            }
            Some((Value::Bool(data[1] != 0), 2))
        }
        0x03 => {
            if data.len() < 3 {
                return None;
            }
            let len = u16::from_le_bytes(data[1..3].try_into().ok()?);
            let total_len = 3 + (len as usize);
            if data.len() < total_len {
                return None;
            }
            let str_bytes = &data[3..total_len];
            let s = String::from_utf8(str_bytes.to_vec()).ok()?;
            Some((Value::Str(s), total_len))
        }
        _ => None,
    }
}
