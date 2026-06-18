use hoi4_radio_maker_lib::audio::analyze_audio;
use std::path::PathBuf;

#[tokio::test]
async fn test_analyze_missing_file_fails() {
    let result = analyze_audio(PathBuf::from("/nonexistent/file.mp3"), None).await;
    assert!(result.is_err());
}
