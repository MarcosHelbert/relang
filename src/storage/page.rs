use super::serializer::{ deserialize_value, serialize_value, Value };

const PAGE_SIZE: usize = 4096;
const HEADER_SIZE: usize = std::mem::size_of::<PageHeader>();
const MAX_OFFSETS: usize = 512;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PageHeader {
    pub page_type: u8,
    pub _reserved: [u8; 3],
    pub page_id: u64,
    pub record_count: u16,
    pub free_space_offset: u16,
    pub checksum: u32, // futuro
}

impl PageHeader {
    pub fn new(page_type: u8, page_id: u64) -> Self {
        Self {
            page_type,
            _reserved: [0; 3],
            page_id,
            record_count: 0,
            free_space_offset: PAGE_SIZE as u16,
            checksum: 0,
        }
    }
}

pub struct Page {
    pub header: PageHeader,
    pub data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new(page_type: u8, page_id: u64) -> Self {
        let header = PageHeader::new(page_type, page_id);
        let mut data = [0u8; PAGE_SIZE];

        // Writes the header at the beginning of the buffer
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &header as *const _ as *const u8,
                std::mem::size_of::<PageHeader>()
            )
        };
        data[..HEADER_SIZE].copy_from_slice(header_bytes);

        Self { header, data }
    }

    pub fn insert_value(&mut self, value: &Value) -> Result<(), &'static str> {
        let encoded = serialize_value(value);
        let record_len = encoded.len();

        if (self.header.record_count as usize) >= MAX_OFFSETS {
            return Err("Limite de registros atingido");
        }

        let offset_table_start = HEADER_SIZE;
        let offset_table_len = (self.header.record_count as usize) * 2;
        let new_offset_pos = offset_table_start + offset_table_len;

        if new_offset_pos + 2 > (self.header.free_space_offset as usize) {
            return Err("Espaço insuficiente para novo offset");
        }

        let data_insert_pos = (self.header.free_space_offset as usize) - record_len;
        if data_insert_pos < new_offset_pos + 2 {
            return Err("Espaço insuficiente para registro");
        }

        // Writes the serialized data
        self.data[data_insert_pos..data_insert_pos + record_len].copy_from_slice(&encoded);

        // Writes the new offset
        let offset_bytes = (data_insert_pos as u16).to_le_bytes();
        self.data[new_offset_pos..new_offset_pos + 2].copy_from_slice(&offset_bytes);

        // Updates header
        self.header.record_count += 1;
        self.header.free_space_offset = data_insert_pos as u16;

        Ok(())
    }

    pub fn read_value(&self, index: usize) -> Option<Value> {
        if index >= (self.header.record_count as usize) {
            return None;
        }

        let offset_pos = HEADER_SIZE + index * 2;
        let offset_bytes = &self.data[offset_pos..offset_pos + 2];
        let offset = u16::from_le_bytes(offset_bytes.try_into().ok()?);

        let slice = &self.data[offset as usize..];
        let (value, _) = deserialize_value(slice)?;
        Some(value)
    }

    pub fn to_raw_buffer(&self) -> [u8; 4096] {
        let mut buffer = [0u8; 4096];

        // Copies the header
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &self.header as *const _ as *const u8,
                std::mem::size_of::<PageHeader>()
            )
        };
        buffer[..HEADER_SIZE].copy_from_slice(header_bytes);

        // Copies the offset table
        buffer[HEADER_SIZE..HEADER_SIZE + (self.header.record_count as usize) * 2].copy_from_slice(
            &self.data[HEADER_SIZE..HEADER_SIZE + (self.header.record_count as usize) * 2]
        );

        // Copies the record data
        buffer[HEADER_SIZE + (self.header.record_count as usize) * 2..].copy_from_slice(
            &self.data[HEADER_SIZE + (self.header.record_count as usize) * 2..]
        );

        buffer
    }

    pub fn from_raw_buffer(buffer: [u8; 4096]) -> Self {
        let header: PageHeader = unsafe { std::ptr::read(buffer.as_ptr() as *const PageHeader) };

        let mut data = [0u8; PAGE_SIZE];
        data.copy_from_slice(&buffer);

        Page { header, data }
    }
}
