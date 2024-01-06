use std::env;
use crate::build_info;

static mut USER_AGENT: Option<String> = None;

pub(crate) fn get_user_agent() -> String {
    unsafe {
        if USER_AGENT.is_none() {
            let version = crate::version();
            let mut repo_url = build_info::PKG_REPOSITORY.to_string();
            if let Some(commit) = build_info::GIT_COMMIT_HASH_SHORT {
                repo_url = format!("{repo_url}/{commit}");
            }
            let os_family = build_info::CFG_FAMILY;
            let mut user_agent = format!("PictureBot/{version} ({os_family};) {repo_url}");
            if let Ok(contact) = env::var("UPLOAD_CONTACT_INFO") {
                user_agent = format!("{user_agent} ({contact})");
            }
            USER_AGENT = Some(user_agent);
        }

        USER_AGENT.clone().unwrap()
    }
}