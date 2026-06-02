
use dotext::MsDoc;
use std::fs;
use std::io::Read;
use std::path::Path;

fn gpu_ocr_fallback(file_path: &str) -> Result<String, String> {
    println!("Running GPU fallback for {}", file_path);
    crate::vision_processor::extract_text_from_image(file_path)
}

pub fn sanitize_output(raw_text: &str) -> String {
    let mut sanitized = String::new();
    let mut consecutive_newlines = 0;

    for line in raw_text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            consecutive_newlines += 1;
        } else {
            if consecutive_newlines > 0 {
                if !sanitized.is_empty() {
                    sanitized.push_str("\n\n");
                }
            } else if !sanitized.is_empty() {
                sanitized.push('\n');
            }
            sanitized.push_str(trimmed);
            consecutive_newlines = 0;
        }
    }

    sanitized
}

pub fn parse_file(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let result = match ext.as_str() {
        "txt" | "md" => fs::read_to_string(path).map_err(|e| e.to_string()),
        "pdf" => match pdf_extract::extract_text(path) {
            Ok(text) => {
                if text.trim().is_empty() {
                    gpu_ocr_fallback(file_path)
                } else {
                    Ok(text)
                }
            }
            Err(e) => Err(format!("Failed to extract text from PDF: {}", e)),
        },
        "docx" => match dotext::Docx::open(path) {
            Ok(mut docx) => {
                let mut text = String::new();
                match docx.read_to_string(&mut text) {
                    Ok(_) => Ok(text),
                    Err(e) => Err(format!("Failed to read text from DOCX: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to open DOCX: {}", e)),
        },
        "png" | "jpg" | "jpeg" => gpu_ocr_fallback(file_path),
        _ => Err(format!("Unsupported file extension: .{}", ext)),
    };

    result.map(|text| sanitize_output(&text))
}

#[cfg(test)]
mod tests {
    use candle_core::{Device, Tensor};

    #[test]
    fn test_cuda_is_working() -> candle_core::Result<()> {
        // 1. Tell Rust to use the primary Nvidia GPU
        let device = Device::new_cuda(0)?;

        // 2. Load two arrays directly into GPU VRAM
        let a = Tensor::new(&[1f32, 2.0, 3.0], &device)?;
        let b = Tensor::new(&[4f32, 5.0, 6.0], &device)?;

        // 3. Perform the math operation on the GPU
        let c = (&a + &b)?;

        // 4. Pull the result back to the CPU to print it
        println!("CUDA Math Success! Result: {:?}", c.to_vec1::<f32>()?);

        Ok(())
    }
}
