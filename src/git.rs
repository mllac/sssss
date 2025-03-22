use color_eyre::eyre::Result;
use git2::{PushOptions, Remote, RemoteCallbacks, Repository};

use crate::cred;

fn commit_tree(r: &Repository) -> Result<()> {
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
                "HEAD",
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
                "HEAD",
                &tree,
                &[],
            )?;
        }
    }

    Ok(())
}

fn add_tree(r: &Repository) -> Result<()> {
    let mut index = r.index()?;

    index.add_all(
        ["."],
        Default::default(),
        None,
    )?;

    Ok(())
}

fn push_tree(
    remote: &mut Remote,
) -> Result<()> {
    let mut cbs = RemoteCallbacks::new();

    cbs.credentials(|_, uname, _| {
        cred::cred(
            uname.expect("invalid remote user")
        )
    });

    let mut opts = PushOptions::new();

    opts.remote_callbacks(cbs);

    let refs = remote.refspecs()
        .filter_map(|x| x.str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();

    remote.push::<String>(
        &refs,
        Some(&mut opts),
    )?;

    println!("NOTHERE");

    Ok(())
}

pub fn update(
    repo: Repository,
    remote_url: &str,
) -> Result<()> {
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

    add_tree(&repo)?;
    commit_tree(&repo)?;
    push_tree(&mut remote)?;

    Ok(())
}
