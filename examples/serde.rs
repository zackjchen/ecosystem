use anyhow::Result;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct User {
    name: String,
    age: u8,
    skills: Vec<String>,
}
fn main() -> Result<()> {
    let user = User {
        name: "John".to_string(),
        age: 25,
        skills: vec!["Rust".to_string(), "Python".to_string()],
    };
    let json = serde_json::to_string(&user)?;
    println!("{}", json);
    let user: User = serde_json::from_str(&json)?;
    print!("{:?}", user);
    Ok(())
}
