use regex::Regex;

pub struct Doc{
    title: String,
    signature: String,
    description: String,
}

pub fn create_docs(path : String) -> Result<(),String>
{
    let source = std::fs::read_to_string(std::path::PathBuf::from(&path)).map_err(|e|println!("{e}")).unwrap();
    
    let matches = Regex::new(r"@([\s\S]*?)@").unwrap();
    let comment_iter = matches.find_iter(&source);
    let mut content = String::new();
    for m in comment_iter {
        let parts : Vec<&str> = m.as_str().split(":").collect();

        let doc = Doc {
            title: String::from(parts[0])[1..].to_string(),
            signature: String::from(parts[1]),
            description: if parts.len() > 2  {String::from(parts[2]).trim()[..(parts[2].len() - 1)].to_string()} else {String::from("")},
        };
        content.push_str(&mut format!("{}:{}\n{}", doc.title, doc.signature, doc.description));
        
    }
    std::fs::write("./docs.txt", content).unwrap();
    Ok(())
}