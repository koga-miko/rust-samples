mod git;

fn main() {
    let path = std::env::args_os().skip(1).next().expect("Usage: my-git2-rs <path>");
    let repo = git::Repository::open(path).expect("opening repository");
    let commit_oid = repo.reference_name_to_id("HEAD").expect("looking up 'HEAD' reference");
    let commit = repo.find_commit(&commit_oid).expect("looking up commit");
    let author = commit.author();
    println!("Author: {} <{}>", author.name().unwrap_or("<no name>"), author.email().unwrap_or("<no email>"));
    println!("Message: {}", commit.message().unwrap_or("<no message>"));
}
