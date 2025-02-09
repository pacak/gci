use std::process::exit;

use git2::Repository;
use inquire::error::InquireError;
use inquire::Select;

#[derive(Debug)]
struct Opt {
    /// Filters remote branches
    remote: bool,
}

fn options() -> Opt {
    use bpaf::*;
    let remote = short('r')
        .long("remote")
        .help("Filters remote branches")
        .switch();

    construct!(Opt { remote })
        .to_options()
        .descr("A rust interactive git checkout CLI.")
        .run()
}

fn main() {
    let args = options();

    let repo = open_curr_repo();

    let branches = if args.remote {
        get_branches_remote(&repo)
    } else {
        get_branches_local(&repo)
    };

    let branch = pp_branches(&branches);

    checkout_branch(&repo, branch)
}

fn open_curr_repo() -> Repository {
    match Repository::discover(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    }
}

fn get_branches_remote(repo: &Repository) -> Vec<(git2::Branch, git2::BranchType)> {
    match repo.branches(Some(git2::BranchType::Remote)) {
        Ok(branches) => branches.filter_map(|b| b.ok()).collect(),
        Err(_) => panic!("failed to get branches"),
    }
}

fn get_branches_local(repo: &Repository) -> Vec<(git2::Branch, git2::BranchType)> {
    match repo.branches(Some(git2::BranchType::Local)) {
        Ok(branches) => branches.filter_map(|b| b.ok()).collect(),
        Err(_) => panic!("failed to get branches"),
    }
}

fn pp_branches<'a>(branches: &'a [(git2::Branch, git2::BranchType)]) -> &'a str {
    let items = branches
        .iter()
        .map(|b| b.0.name().unwrap().unwrap())
        .collect::<Vec<_>>();
    let mut select: Select<&str> = Select::new("Select a git branch:", items);

    select.vim_mode = true;

    let ans: Result<&str, InquireError> = select.prompt();

    match ans {
        Ok(ans) => ans,
        // gracefully shutdown
        Err(_) => exit(0),
    }
}

fn checkout_branch(repo: &Repository, branch_name: &str) {
    let (object, reference) = repo.revparse_ext(branch_name).expect("Object not found");

    repo.checkout_tree(&object, None)
        .expect("Failed to checkout");

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repo.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repo.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");
}
