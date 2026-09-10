use hoi4_radio_maker_lib::db::Db;
use hoi4_radio_maker_lib::models::CreateProjectRequest;
use rusqlite::Connection;
use tempfile::TempDir;

#[test]
fn test_project_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("app.db");
    let db = Db::open(&db_path).unwrap();

    let req = CreateProjectRequest {
        name: "Test Project".to_string(),
        version: "0.1.0".to_string(),
        supported_version: "1.14.*".to_string(),
        tags: vec!["test".to_string()],
        author: Some("Tester".to_string()),
        output_dir: tmp.path().join("out"),
    };

    let created = db.create_project(&req).unwrap();
    assert_eq!(created.name, "Test Project");
    assert!(created.id.starts_with("proj_"));

    let fetched = db.get_project(&created.id).unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "Test Project");

    let projects = db.list_projects().unwrap();
    assert_eq!(projects.len(), 1);

    db.delete_project(&created.id).unwrap();

    let projects = db.list_projects().unwrap();
    assert!(projects.is_empty());
    assert!(db.get_project(&created.id).unwrap().is_none());
}

/// A database already at user_version 2 (stations without the `subdir` column,
/// holding rows) must gain the column on open and keep its data.
#[test]
fn migration_v2_to_v3_adds_subdir_and_preserves_rows() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("app.db");

    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE projects (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL,
                supported_version TEXT NOT NULL, tags TEXT NOT NULL, author TEXT,
                output_dir TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );
            CREATE TABLE stations (
                id TEXT PRIMARY KEY, project_id TEXT NOT NULL, name TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO stations (id, project_id, name) VALUES ('legacy_station', 'p1', 'Legacy');
            PRAGMA user_version = 2;
            ",
        )
        .unwrap();
        let has_subdir: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('stations') WHERE name = 'subdir'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            has_subdir, 0,
            "precondition: v2 schema has no subdir column"
        );
    }

    let db = Db::open(&path).unwrap();

    let version: i32 = db
        .conn()
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 3, "migration must bump user_version to 3");

    let (id, name, subdir): (String, String, Option<String>) = db
        .conn()
        .query_row(
            "SELECT id, name, subdir FROM stations WHERE id = 'legacy_station'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(id, "legacy_station");
    assert_eq!(name, "Legacy");
    assert!(subdir.is_none(), "legacy rows default to a NULL subdir");
}
