use std::{error::Error, fs, path::Path};

pub(crate) fn prepare(root: &Path) -> Result<(), Box<dyn Error>> {
    let files = [
        ("example.rs", "fn main(){println!(\"hello\");}\n"),
        ("example.js", "const message={hello:'world'};\n"),
        ("example.jsx", "const App=()=> <div>Hello</div>;\n"),
        ("example.ts", "const message:string='hello';\n"),
        ("example.tsx", "const App=()=> <div>Hello</div>;\n"),
        ("example.json", "{\"hello\":\"world\",\"value\":42}\n"),
        (
            "example.jsonc",
            "{ \"hello\": \"world\", // comment\n \"value\": 42 }\n",
        ),
        ("example.css", "body{color:red;background:white}\n"),
        ("example.graphql", "query GetUser{user{id name}}\n"),
        ("example.gql", "query GetUsers{users{id name}}\n"),
        ("example.txt", "This file must not be formatted.\n"),
    ];

    for (name, content) in files {
        fs::write(root.join(name), content)?;
    }

    Ok(())
}
