use std::path::PathBuf;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let url = "https://github.com/dakychan/Aporia/releases/download/0.5.0/Aporia.client.jar";
    let output = "test_aporia.jar";
    
    println!("Testing download from: {}", url);
    println!("Output: {}", output);
    
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap();
    
    match client.get(url).send().await {
        Ok(response) => {
            println!("Response status: {}", response.status());
            
            if !response.status().is_success() {
                println!("Error: HTTP {}", response.status());
                return;
            }
            
            match response.bytes().await {
                Ok(bytes) => {
                    println!("Downloaded {} bytes", bytes.len());
                    
                    if bytes.is_empty() {
                        println!("Error: File is empty!");
                        return;
                    }
                    
                    match std::fs::write(output, &bytes) {
                        Ok(_) => println!("File saved successfully"),
                        Err(e) => println!("Error saving file: {}", e),
                    }
                }
                Err(e) => println!("Error reading bytes: {}", e),
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
