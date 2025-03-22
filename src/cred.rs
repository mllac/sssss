use std::{env, path::PathBuf};

use git2::Cred;

const ID_ED25519: &str = "id_ed25519";
const ID_RSA: &str = "id_rsa";

fn ssh_key() -> PathBuf {
    let home = env::home_dir()
        .expect("failed to get home dir");

    home
        .join(".ssh")
        .join(ID_ED25519)
}

pub fn cred(
    username: &str,
) -> Result<Cred, git2::Error> {
    Cred::ssh_key(
        username,
        None,
        &ssh_key(),
        None,
    )
}
