use tokio::io::{self, AsyncBufReadExt, BufReader};
use serde_json::{json, Value};
use std::io::Write;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("Starting MarkItDown MCP Server");

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    while let Some(line) = reader.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                continue;
            }
        };

        if let Some(id) = req.get("id") {
            let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
            
            let result = match method {
                "initialize" => {
                    json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": "markitdown-mcp",
                            "version": "0.1.0"
                        }
                    })
                }
                "tools/list" => {
                    json!({
                        "tools": [
                            {
                                "name": "read_and_convert_document",
                                "description": "Read a document and convert it to markdown.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "absolute_file_path": {
                                            "type": "string",
                                            "description": "Absolute path to the document file"
                                        }
                                    },
                                    "required": ["absolute_file_path"]
                                }
                            }
                        ]
                    })
                }
                "tools/call" => {
                    if let Some(params) = req.get("params") {
                        let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        if tool_name == "read_and_convert_document" {
                            if let Some(args) = params.get("arguments") {
                                if let Some(path_val) = args.get("absolute_file_path") {
                                    if let Some(path_str) = path_val.as_str() {
                                        let text_result = process_file(path_str);
                                        json!({
                                            "content": [
                                                {
                                                    "type": "text",
                                                    "text": text_result
                                                }
                                            ]
                                        })
                                    } else {
                                        json!({ "isError": true, "content": [{"type": "text", "text": "absolute_file_path must be a string"}] })
                                    }
                                } else {
                                    json!({ "isError": true, "content": [{"type": "text", "text": "Missing absolute_file_path argument"}] })
                                }
                            } else {
                                json!({ "isError": true, "content": [{"type": "text", "text": "Missing arguments"}] })
                            }
                        } else {
                            json!({ "isError": true, "content": [{"type": "text", "text": format!("Unknown tool: {}", tool_name)}] })
                        }
                    } else {
                         json!({ "isError": true, "content": [{"type": "text", "text": "Missing params"}] })
                    }
                }
                _ => {
                    // Unhandled method
                    json!({})
                }
            };

            let response = json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            });

            // Write to stdout and flush immediately
            println!("{}", serde_json::to_string(&response).unwrap());
            std::io::stdout().flush().unwrap();
        } else {
            // It's a notification
            // e.g. "notifications/initialized"
        }
    }

    Ok(())
}

fn process_file(path_str: &str) -> String {
    match markitdown_native_lib::parser::parse_file(path_str) {
        Ok(content) => content,
        Err(e) => format!("Error parsing file: {}", e)
    }
}
