use storage::{ manager::PageManager, serializer::Value };

pub mod storage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new PageManager
    let mut manager = PageManager::new("my_database.db")?;

    // Create a new page
    let mut page = manager.create_page(0)?;

    // Insert a value into the page
    let value = Value::Str("Hello, World!".to_string());
    page.insert_value(&value)?;

    // Write the page to the file
    manager.write_page(&page)?;

    // Read the page back from the file
    let page = manager.read_page(0)?;
    if let Some(read_value) = page.read_value(0) {
        println!("{:?}", read_value);
    }

    // Close the PageManager
    manager.close()?;

    Ok(())
}
