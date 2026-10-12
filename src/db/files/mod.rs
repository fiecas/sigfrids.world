use postgres::{Client, Error};

pub struct File {
    id: String,
    original_name: String,
    content_type: Option<String>,
    size: i64,
}

pub fn new(
    mut client: Client,
    original_name: &str,
    content_type: &str,
    size: i64,
) -> Result<File, Error> {
    let rows = client.query(
        include_str!("./sql/insert.sql"),
        &[&original_name, &content_type, &size],
    )?;
    let row = rows.get(0).unwrap();

    let id = row.get("id");

    Ok(File {
        id: id,
        original_name: original_name.to_string(),
        content_type: Some(content_type.to_string()),
        size,
    })
}

pub fn get(mut client: Client, id: &str) -> Result<Option<File>, Error> {
    let rows = client.query(include_str!("./sql/select_all_where_id.sql"), &[&id])?;
    let row = match rows.get(0) {
        Some(row) => row,
        None => return Ok(None),
    };

    let id = row.get("id");
    let original_name = row.get("original_name");
    let content_type = row.get("content_type");
    let size = row.get("size");

    Ok(Some(File {
        id,
        original_name,
        content_type,
        size,
    }))
}

pub fn delete(mut client: Client, id: &str) -> Result<(), Error> {
    client.execute(include_str!("./sql/delete.sql"), &[&id])?;

    Ok(())
}
