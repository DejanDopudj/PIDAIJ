use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Value {
    Integer(i32),
    Double(f64),
    String(String),
}

struct DynamicThreadPoolServer {
    host: String,
    port: u16,
    dictionary: Arc<RwLock<HashMap<String, Value>>>,
}

impl DynamicThreadPoolServer {
    fn new(host: &str, port: u16) -> Self {
        DynamicThreadPoolServer {
            host: host.to_string(),
            port,
            dictionary: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn start_server(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await.unwrap();
        println!("Server listening on {}:{}", self.host, self.port);
        while let Ok((stream, _)) = listener.accept().await {
            let dictionary = Arc::clone(&self.dictionary);
            tokio::spawn(async move {
                Self::handle_client_with_dictionary(stream, dictionary).await;
            });
        }
    }

    async fn handle_client_with_dictionary(mut stream: TcpStream, dictionary: Arc<RwLock<HashMap<String, Value>>>) {
        let mut buffer = [0; 1024];
        if let Ok(_) = stream.read(&mut buffer).await {
            let request = String::from_utf8_lossy(&buffer[..]);

            let (response_body, status_line) = if request.starts_with("PUT") {
                if let Some(body) = request.split("\r\n\r\n").nth(1) {
                    let trimmed_body = body.trim();
                    let res_option = trimmed_body.split('}').next();
                    if let Some(res) = res_option {  
                        let mut owned_res = res.to_owned();
                        owned_res.push('}');
                        let json: serde_json::Value = serde_json::from_str(&owned_res)
                            .expect("JSON was not well-formatted");

                        if let (Some(key), Some(value)) = (json.get("key"), json.get("value")) {
                            let key = key.as_str().unwrap().to_string();
                            let value = match value {
                                serde_json::Value::String(s) => Value::String(s.to_string()),
                                serde_json::Value::Number(n) if n.is_i64() => Value::Integer(n.as_i64().unwrap() as i32),
                                serde_json::Value::Number(n) => Value::Double(n.as_f64().unwrap()),
                                _ => Value::String("NULL".to_string()),
                            };
                
                            Self::put_with_dictionary(&dictionary, key, value);
                            ("PUT successful".to_string(), "HTTP/1.1 200 OK")
                        } else {
                            ("Invalid JSON".to_string(), "HTTP/1.1 400 Bad Request")
                        }
                    } else {
                        ("Invalid JSON".to_string(), "HTTP/1.1 400 Bad Request")
                    }
                } else {
                    ("Invalid request body".to_string(), "HTTP/1.1 400 Bad Request")
                }
            }
            else if request.starts_with("GET") {
                if let Some(key) = request.split("?").nth(1).unwrap_or("").split(" ").next() {
                    let value = Self::get_with_dictionary(&dictionary, key.split("=").nth(1).unwrap_or(""));
                    (format!("GET result: {:?}", value), "HTTP/1.1 200 OK")
                } else {
                    ("Invalid GET request".to_string(), "HTTP/1.1 400 Bad Request")
                }
            } else {
                ("Invalid request".to_string(), "HTTP/1.1 400 Bad Request")
            };

            let response_headers = "Content-Type: text/plain\r\n";
            let response = format!("{}\r\n{}\r\n\r\n{}", status_line, response_headers, response_body);

            if let Err(_) = stream.write(response.as_bytes()).await {
                eprintln!("Error writing response to stream");
            }
        }
    }

    fn put_with_dictionary(dictionary: &Arc<RwLock<HashMap<String, Value>>>, key: String, value: Value) {
        let mut dictionary = dictionary.write().unwrap();
        dictionary.insert(key, value);
    }

    fn get_with_dictionary(dictionary: &Arc<RwLock<HashMap<String, Value>>>, key: &str) -> Value {
        let dictionary = dictionary.read().unwrap();
        dictionary.get(key).cloned().unwrap_or(Value::String("NULL".to_string()))
    }

    fn print_dict_info_with_dictionary(dictionary: &Arc<RwLock<HashMap<String, Value>>>) {
        for (key, value) in dictionary.read().unwrap().iter() {
            println!("Key: {}", key);
            println!("Value: {:?}", value);
            println!("Type of Value: {:?}", std::mem::discriminant(value));
            println!("------");
        }
    }
}

#[tokio::main]
async fn main() {
    let server = DynamicThreadPoolServer::new("127.0.0.1", 8082);
    server.start_server().await;
}
