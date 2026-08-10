#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GitWorkspaceSummary {
    pub(crate) changed_files: usize,
    pub(crate) staged_files: usize,
    pub(crate) unstaged_files: usize,
    pub(crate) untracked_files: usize,
    pub(crate) conflicted_files: usize,
}

impl GitWorkspaceSummary {
    pub(crate) fn is_clean(self) -> bool {
        self.changed_files == 0
    }

    pub(crate) fn headline(self) -> String {
        if self.is_clean() {
            "clean".to_string()
        } else {
            let mut details = Vec::new();
            if self.staged_files > 0 {
                details.push(format!("{} staged", self.staged_files));
            }
            if self.unstaged_files > 0 {
                details.push(format!("{} unstaged", self.unstaged_files));
            }
            if self.untracked_files > 0 {
                details.push(format!("{} untracked", self.untracked_files));
            }
            if self.conflicted_files > 0 {
                details.push(format!("{} conflicted", self.conflicted_files));
            }
            format!(
                "dirty · {} files · {}",
                self.changed_files,
                details.join(", ")
            )
        }
    }
}
