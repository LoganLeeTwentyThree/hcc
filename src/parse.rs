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
    for m in comment_iter {
        let parts : Vec<&str> = m.as_str().split(":").collect();
        
        let doc = Doc {
            title: String::from(parts[0]),
            signature: String::from(parts[1]),
            description: if parts.len() > 2  {String::from(parts[2])} else {String::from("")},
        };
        serialize_doc("./docs.txt".to_string(), doc).unwrap()
    }
    Ok(())
}

fn serialize_doc(path : String, doc : Doc) -> Result<(),String>
{

    let contents = 
    format!("{}:{}\n{}\n", doc.title,doc.signature, doc.description);
    std::fs::write(path, contents).unwrap();
    Ok(())
}