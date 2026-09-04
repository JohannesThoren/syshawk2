use pam::Client;
use users::os::unix::GroupExt;

/// Verifies a username/password against the host's PAM stack (the same
/// system accounts used to log into the machine over SSH/console).
/// Requires the process to have permission to read shadow entries -
/// in practice this means running as root or with the `shadow` group.
pub fn verify_password(username: &str, password: &str) -> bool {
    let Ok(mut client) = Client::with_password("login") else {
        tracing::error!("failed to initialize PAM client");
        return false;
    };
    client.conversation_mut().set_credentials(username, password);
    client.authenticate().is_ok()
}

/// True if `username` is a member of `group_name`, checking both the
/// user's primary group and supplementary group membership.
pub fn is_member_of(username: &str, group_name: &str) -> bool {
    let Some(group) = users::get_group_by_name(group_name) else {
        tracing::warn!(group = group_name, "group does not exist on this host");
        return false;
    };
    if group
        .members()
        .iter()
        .any(|m| m.to_string_lossy() == username)
    {
        return true;
    }
    let Some(user) = users::get_user_by_name(username) else {
        return false;
    };
    user.primary_group_id() == group.gid()
}
