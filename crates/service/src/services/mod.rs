// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
pub mod calendar;
pub mod dashboard;
pub mod github;
pub mod gitlab;
pub mod jira;

/// Strip protocol prefix and trailing slashes from a host string.
/// Shared utility used by GitLab and Jira services.
pub fn sanitize_host(host: &str) -> String {
    host.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}
