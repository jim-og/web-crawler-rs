use url::Url;

use std::{collections::HashSet, io};

#[derive(Default)]
pub struct Printer {}

impl Printer {
    pub fn print(mut wrt: impl io::Write, url: Url, links: HashSet<Url>) -> io::Result<()> {
        let mut buffer = String::new();

        buffer.push_str(&format!("{}\n", url));
        for link in links {
            buffer.push_str(&format!("--{}\n", link));
        }

        writeln!(wrt, "{}", buffer)?;
        Ok(())
    }
}
