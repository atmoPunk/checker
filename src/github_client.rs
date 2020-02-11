pub fn get_commits(owner: &str, repo: &str) -> Result<serde_json::value::Value, std::io::Error> {
    let resp = ureq::get(&format!("https://api.github.com/repos/{}/{}/commits", owner, repo)).call();

    if resp.ok() {
        //let mut json: Result<serde_json::value::Value, std::io::Error> = resp.into_json();
        resp.into_json()
    } else {
        eprintln!("{}", resp.status_line());
        Err(std::io::Error::new(std::io::ErrorKind::Other, "not OK"))
    }
}