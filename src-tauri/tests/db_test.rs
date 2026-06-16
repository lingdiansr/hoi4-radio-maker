use hoi4_radio_maker_lib::db::Db;
use hoi4_radio_maker_lib::models::CreateProjectRequest;
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
