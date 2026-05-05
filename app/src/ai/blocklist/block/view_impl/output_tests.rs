use ai::agent::action::UploadArtifactRequest;

use crate::ai::agent::UploadArtifactResult;

use super::format_upload_artifact_text;

#[test]
fn format_upload_artifact_text_includes_request_details() {
    let request = UploadArtifactRequest {
        file_path: "reports/daily.txt".to_string(),
        description: Some("Daily summary".to_string()),
    };

    let text = format_upload_artifact_text(&request, None);

    assert_eq!(
        text,
        "上传 artifact：reports/daily.txt\n描述：Daily summary"
    );
}

#[test]
fn format_upload_artifact_text_includes_success_summary() {
    let request = UploadArtifactRequest {
        file_path: "reports/daily.txt".to_string(),
        description: Some("Daily summary".to_string()),
    };
    let result = UploadArtifactResult::Success {
        artifact_uid: "artifact-123".to_string(),
        filepath: Some("reports/daily.txt".to_string()),
        mime_type: "text/plain".to_string(),
        description: Some("Daily summary".to_string()),
        size_bytes: 128,
    };

    let text = format_upload_artifact_text(&request, Some(&result));

    assert_eq!(
        text,
        "上传 artifact：reports/daily.txt\n描述：Daily summary\n状态：已上传 artifact artifact-123\n已上传文件：reports/daily.txt"
    );
}

#[test]
fn format_upload_artifact_text_includes_terminal_status() {
    let request = UploadArtifactRequest {
        file_path: "reports/daily.txt".to_string(),
        description: None,
    };

    let error_text = format_upload_artifact_text(
        &request,
        Some(&UploadArtifactResult::Error(
            "permission denied".to_string(),
        )),
    );
    assert_eq!(
        error_text,
        "上传 artifact：reports/daily.txt\n状态：上传失败：permission denied"
    );

    let cancelled_text =
        format_upload_artifact_text(&request, Some(&UploadArtifactResult::Cancelled));
    assert_eq!(cancelled_text, "上传 artifact：reports/daily.txt");
}
