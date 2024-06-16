use anyhow::Result;
use chrono::{DateTime, Datelike as _, Utc};
use derive_builder::Builder;
fn main() -> Result<()> {
    let user = User::build()
        .name("Alice")
        .skill("Rust".to_string())
        .skill("Python".to_string())
        .date("1999-06-12 19:30:00+00:00")
        .build()?;
    println!("{:?}", user);
    Ok(())
}

#[allow(unused)]
#[derive(Debug, Builder)]
#[builder(pattern = "owned")] // consume and return owned values
#[builder(build_fn(name = "_priv_build", private))]
struct User {
    #[builder(setter(into))]
    name: String,
    #[builder(setter(skip = true))]
    age: u32,
    #[builder(setter(into), default = "\"male\"")]
    sex: &'static str,
    #[builder(setter(each(name = "skill", into)), default = "vec![]")]
    skills: Vec<String>,
    #[builder(setter(strip_option), default = "None")]
    email: Option<String>,

    //这样直接into是不行的
    // #[builder(setter(into))]
    #[builder(setter(custom), default = "Utc::now()")]
    date: DateTime<Utc>,
}

impl User {
    fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

impl UserBuilder {
    pub fn build(self) -> Result<User> {
        let mut user = self._priv_build()?;
        user.age = (Utc::now().year() - user.date.year()) as u32;
        Ok(user)
    }

    pub fn date(mut self, date: &'static str) -> Self {
        self.date = DateTime::parse_from_rfc3339(date)
            .map(|dt| dt.with_timezone(&Utc))
            .ok();
        self
    }
}
