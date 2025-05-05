use std::fs::{ File, OpenOptions };
use std::io::{ Read, Write, Seek, SeekFrom };
use std::path::Path;
use std::error::Error;

use super::page::Page;

pub struct PageManager {
    pub file: File,
    pub page_count: u64,
}

impl PageManager {
    // Creation of the page manager
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = if path.exists() {
            OpenOptions::new().read(true).write(true).open(path)?
        } else {
            OpenOptions::new().create(true).write(true).open(path)?
        };

        let page_count = file.metadata()?.len() / 4096;
        Ok(PageManager { file, page_count })
    }

    // Reading a page from the file
    pub fn read_page(&mut self, page_id: u32) -> Result<Page, Box<dyn Error>> {
        let position = (page_id as u64) * 4096; // Offset do arquivo
        self.file.seek(SeekFrom::Start(position))?;

        let mut buffer = [0u8; 4096];
        self.file.read_exact(&mut buffer)?;

        // A partir do buffer lido, recria a página
        let page = Page::from_raw_buffer(buffer);

        Ok(page)
    }

    // Writing a page to the file
    pub fn write_page(&mut self, page: &Page) -> Result<(), Box<dyn Error>> {
        let position = (page.header.page_id as u64) * 4096; // Offset do arquivo
        self.file.seek(SeekFrom::Start(position))?;

        let buffer = page.to_raw_buffer();
        self.file.write_all(&buffer)?;

        Ok(())
    }

    // Creation of a new page and insertion
    pub fn create_page(&mut self, page_type: u8) -> Result<Page, Box<dyn Error>> {
        let page_id = self.page_count;
        self.page_count += 1;

        let page = Page::new(page_type, page_id);
        self.write_page(&page)?;

        Ok(page)
    }

    // Close the file (save changes)
    pub fn close(&mut self) -> Result<(), Box<dyn Error>> {
        self.file.flush()?;
        Ok(())
    }
}
