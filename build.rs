use std::env;

// check if `input` is in $PATH
fn in_path(input: &str) -> bool {
    env::var_os("PATH")
        .as_deref()
        .map(env::split_paths)
        .map(|mut x| {
            x.any(|x| {
                x.join(input)
                    .is_file()
            })
        })
        .unwrap_or(false)
}

fn check_dependencies() {
    let Some(p) = [
        "git",
    ]
    .into_iter()
    .find(|x| !in_path(x)) else {
        return;
    };

    panic!("missing dependency: '{}'", p);
}

fn main() {
    check_dependencies();
}
