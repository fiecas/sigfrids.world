use include_dir::{Dir, include_dir};
use postgres::Client;

static MIGRATIONS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/migrations");

pub fn migrate(client: &mut Client) -> Result<(), postgres::Error> {
    client.batch_execute(
        "--sql
        CREATE TABLE IF NOT EXISTS schema_migrations (
            id TEXT PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
        )
    ",
    )?;

    let mut files: Vec<_> = MIGRATIONS.files().collect();
    files.sort_by_key(|file| file.path());

    for migration in files {
        let id = migration.path().file_name().unwrap().to_str().unwrap();
        let sql = migration.contents_utf8().unwrap();

        let is_applied = client
            .query_opt(
                "--sql
            SELECT 1 FROM schema_migrations WHERE id = $1
        ",
                &[&id],
            )?
            .is_some();
        if is_applied {
            continue;
        }

        let mut tx = client.transaction()?;
        tx.batch_execute(sql)?;
        tx.execute(
            "--sql
        INSERT INTO schema_migrations (id) values ($1)
        ",
            &[&id],
        )?;
        tx.commit()?;
    }

    Ok(())
}
