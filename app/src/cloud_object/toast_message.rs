use warpui::AppContext;

use crate::server::cloud_objects::update_manager::{
    InitiatedBy, ObjectOperation, OperationSuccessType,
};

use super::{CloudObject, GenericStringObjectFormat, JsonObjectType, ObjectType};

pub struct CloudObjectToastMessage;

impl CloudObjectToastMessage {
    pub fn toast_message(
        object: &dyn CloudObject,
        operation: &ObjectOperation,
        success_type: &OperationSuccessType,
        app: &AppContext,
    ) -> Option<String> {
        let object_name = object.model_type_name().to_owned();
        let object_name_lowercase = object_name.to_ascii_lowercase();

        match (object.object_type(), operation, success_type) {
            // We should only show toasts for creates initiated by the user, not by the system
            (
                _,
                ObjectOperation::Create {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Success,
            ) => {
                let containing_object_name = object.containing_object_name(app);
                Some(format!("{object_name} 已保存到 {containing_object_name}"))
            }
            // notebooks intentionally do not have an update message, as they are updated
            // as the user types and so toasts would be VERY noisy
            (ObjectType::Notebook, ObjectOperation::Update, OperationSuccessType::Success) => None,
            (_, ObjectOperation::Update, OperationSuccessType::Success) => {
                Some(format!("{object_name} 已更新"))
            }
            (_, ObjectOperation::MoveToFolder, OperationSuccessType::Success)
            | (_, ObjectOperation::MoveToDrive, OperationSuccessType::Success) => {
                let containing_object_name = object.containing_object_name(app);
                Some(format!("{object_name} 已移动到 {containing_object_name}"))
            }
            (_, ObjectOperation::Trash, OperationSuccessType::Success) => {
                Some(format!("{object_name} 已移到废纸篓"))
            }
            (_, ObjectOperation::Untrash, OperationSuccessType::Success) => {
                Some(format!("{object_name} 已恢复"))
            }
            (_, ObjectOperation::Leave, OperationSuccessType::Success) => {
                Some(format!("已离开 {object_name}"))
            }
            (
                _,
                ObjectOperation::Create {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Failure,
            ) => Some(format!("创建 {object_name_lowercase} 失败")),
            (
                _,
                ObjectOperation::Create {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Denied(message),
            ) => Some(message.to_string()),
            (_, ObjectOperation::Update, OperationSuccessType::Failure) => {
                Some(format!("更新 {object_name_lowercase} 失败"))
            }
            (_, ObjectOperation::MoveToFolder, OperationSuccessType::Failure)
            | (_, ObjectOperation::MoveToDrive, OperationSuccessType::Failure) => {
                Some(format!("移动 {object_name_lowercase} 失败"))
            }
            (_, ObjectOperation::Trash, OperationSuccessType::Failure) => {
                Some(format!("将 {object_name_lowercase} 移到废纸篓失败"))
            }
            (_, ObjectOperation::Untrash, OperationSuccessType::Failure) => {
                Some(format!("恢复 {object_name_lowercase} 失败"))
            }
            // We should only show deletion failure toasts for user-initiated deletions.
            (
                _,
                ObjectOperation::Delete {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Failure,
            ) => Some(format!("删除 {object_name_lowercase} 失败")),
            (_, ObjectOperation::Leave, OperationSuccessType::Failure) => {
                Some(format!("离开 {object_name} 失败"))
            }
            (ObjectType::Workflow, ObjectOperation::Update, OperationSuccessType::Rejection) => {
                Some("无法保存此工作流，因为你编辑期间发生了其他更改。".to_string())
            }
            (
                ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
                    JsonObjectType::EnvVarCollection,
                )),
                ObjectOperation::Update,
                OperationSuccessType::Rejection,
            ) => Some("无法保存环境变量，因为你编辑期间发生了其他更改。".to_string()),
            (
                ObjectType::GenericStringObject(GenericStringObjectFormat::Json(
                    JsonObjectType::AIFact,
                )),
                ObjectOperation::Update,
                OperationSuccessType::Rejection,
            ) => Some("无法保存规则，因为你编辑期间发生了其他更改。".to_string()),
            (_, ObjectOperation::TakeEditAccess, OperationSuccessType::Failure) => {
                Some(format!("开始编辑 {object_name_lowercase} 失败"))
            }
            (_, ObjectOperation::UpdatePermissions, OperationSuccessType::Success) => {
                Some(format!("已成功更新 {object_name_lowercase} 的权限"))
            }
            (_, ObjectOperation::UpdatePermissions, OperationSuccessType::Failure) => {
                Some(format!("更新 {object_name_lowercase} 的权限失败"))
            }
            _ => None,
        }
    }

    pub fn toast_deletion_confirm_message(
        num_objects: i32,
        operation: &ObjectOperation,
        success_type: &OperationSuccessType,
    ) -> Option<String> {
        let count_objects_message = match num_objects {
            1 => "1 个对象".to_string(),
            n => {
                format!("{n} 个对象")
            }
        };
        match (operation, success_type) {
            // We should only show deletion failure toasts for user-initiated deletions.
            (
                ObjectOperation::Delete {
                    initiated_by: InitiatedBy::User,
                },
                OperationSuccessType::Success,
            ) => Some(format!("{count_objects_message}已永久删除")),
            (ObjectOperation::EmptyTrash, OperationSuccessType::Success) => {
                Some(format!("废纸篓已清空：{count_objects_message}已永久删除"))
            }
            (ObjectOperation::EmptyTrash, OperationSuccessType::Failure) => {
                Some("清空废纸篓失败".to_string())
            }
            (ObjectOperation::EmptyTrash, OperationSuccessType::Rejection) => {
                Some("废纸篓中没有可清空的对象".to_string())
            }
            _ => None,
        }
    }
}
