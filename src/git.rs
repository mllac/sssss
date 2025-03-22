use git2::{
    RemoteCallbacks,
    PushOptions,
    Repository,
    Remote,
};
use thiserror::Error;

use crate::cred;

#[derive(
    Error,
    Debug,
)]

pub enum SyncError {
    #[error("failed to commit tree: {0}")]
    Commit(git2::Error),
    #[error("failed to push tree: {0}")]
    Push(git2::Error),
    #[error("failed to add tree: {0}")]
    Add(git2::Error),
    #[error("a git error occurred: {0}")]
    Other(#[from] git2::Error),
}

fn commit_tree(r: &Repository) -> Result<(), git2::Error> {
    let mut index = r.index()?;

    let oid = index.write_tree()?;
    index.write()?;

    let revres = r.revparse_ext("HEAD");
    let tree = r.find_tree(oid)?;
    let sig = r.signature()?;

    match revres.ok() {
        Some((ref x, _)) if let Some(c) = x.as_commit() => {
            r.commit(
                Some("HEAD"),
                &sig, 
                &sig,
                "sync",
                &tree,
                &[c],
            )?;
        },
        Some(_) |
        None => {
            r.commit(
                Some("HEAD"),
                &sig, 
                &sig,
                "sync",
                &tree,
                &[],
            )?;
        }
    }

    Ok(())
}

fn add_tree(r: &Repository) -> Result<(), git2::Error> {
    let mut index = r.index()?;

    index.add_all(
        ["."],
        Default::default(),
        None,
    )
}

fn push_tree(
    remote: &mut Remote,
) -> Result<(), git2::Error> {
    let mut cbs = RemoteCallbacks::new();

    cbs.credentials(|_, uname, _| {
        cred::cred(
            uname.expect("invalid remote user")
        )
    });

    let mut opts = PushOptions::new();

    opts.remote_callbacks(cbs);

    remote.push::<&str>(
        &["refs/heads/sync"],
        Some(&mut opts),
    )?;

    Ok(())
}

pub fn update(
    repo: Repository,
    remote_url: &str,
) -> Result<(), SyncError> {
    let mut remote = repo
        .find_remote("sync").or_else(|_| {
            repo.remote("sync", remote_url)
        })?;

    if remote.url().is_none_or(|x| {
        x != remote_url
    }) {
        repo.remote_set_url(
            "sync",
            remote_url,
        )?;
    }

    add_tree(&repo).map_err(SyncError::Add)?;
    commit_tree(&repo).map_err(SyncError::Commit)?;
    push_tree(&mut remote).map_err(SyncError::Push)?;

    Ok(())
}
