use hoi4_radio_maker_lib::models::Project;
use std::path::PathBuf;

#[test]
fn test_project_has_id_and_name() {
    let p = Project {
        id: "proj_1".into(),
        name: "My Radio".into(),
        version: "0.1.0".into(),
        supported_version: "*".into(),
        tags: vec!["Sound".into()],
        author: Some("Alice".into()),
        output_dir: PathBuf::from("/tmp/out"),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert_eq!(p.name, "My Radio");
}
