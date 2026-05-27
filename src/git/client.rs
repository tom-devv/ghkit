use std::{collections::HashSet, path::Path};

use git2::{Commit, Diff, Error, Oid, Repository};

pub struct GhRepo {
    inner: Repository,
}

impl GhRepo {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<GhRepo, Error> {
        let repo = Repository::open(path)?;
        Ok(GhRepo { inner: repo })
    }

    pub fn get_all_commits<'a>(&'a self) -> Result<Vec<Commit<'a>>, Error> {
        let revwalk: Vec<Oid> = match self.inner.revwalk() {
            Ok(mut walk) => {
                if let Err(e) = walk.push_head() {
                    panic!("Failed to push HEAD to revwalk: {}", e);
                }

                walk.into_iter().flatten().collect()
            }
            Err(e) => panic!("Failed to create revwalk: {}", e),
        };

        let commits: Vec<Commit<'a>> = revwalk
            .iter()
            .map(|oid| self.inner.find_commit(*oid).unwrap())
            .collect();

        Ok(commits)
    }

    pub fn iter_commits(&self) -> Result<impl Iterator<Item = Commit<'_>>, git2::Error> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.push_head()?;

        let repo_ref = &self.inner;

        Ok(revwalk
            .flatten()
            .filter_map(move |oid| repo_ref.find_commit(oid).ok()))
    }

    pub fn get_commiters(&self) -> Result<HashSet<String>, git2::Error> {
        let mut authors: HashSet<String> = HashSet::new();
        for commit in self.iter_commits()? {
            let commiter = commit.author();
            if let Ok(email) = commiter.email() {
                authors.insert(email.to_owned());
            }
        }

        Ok(authors)
    }

    pub fn get_author_commits<'a>(
        &'a self,
        email: String,
    ) -> Result<impl Iterator<Item = Commit<'a>> + 'a, git2::Error> {
        let iter = self
            .iter_commits()?
            .filter(move |commit| commit.author().email().map_or(false, |e| e == email));

        Ok(iter)
    }

    pub fn get_diff(
        &self,
        parent: Option<&Commit>,
        current: Option<&Commit>,
    ) -> Result<Diff<'_>, git2::Error> {
        let parent_tree = parent.map(|c| c.tree()).transpose()?;
        let current_tree = current.map(|c| c.tree()).transpose()?;

        let diff =
            self.inner
                .diff_tree_to_tree(parent_tree.as_ref(), current_tree.as_ref(), None)?;
        Ok(diff)
    }

    pub fn get_parent_diff(&self, commit: &Commit) -> Result<Diff<'_>, git2::Error> {
        let parent_commit = match commit.parent(0) {
            Ok(parent) => Some(parent),
            Err(_) => None,
        };

        self.get_diff(parent_commit.as_ref(), Some(commit))
    }
}
