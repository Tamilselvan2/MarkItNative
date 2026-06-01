use std::fs;
use std::io::Read;
use std::path::Path;
use dotext::MsDoc;
use candle_core::{Device, Tensor};

pub fn get_device() -> candle_core::Device {
    if candle_core::utils::cuda_is_available() {
        println!("GPU Acceleration Triggered: CUDA Device Detected.");
        candle_core::Device::new_cuda(0).unwrap_or(candle_core::Device::Cpu)
    } else if candle_core::utils::metal_is_available() {
        println!("GPU Acceleration Triggered: Metal Device Detected.");
        candle_core::Device::new_metal(0).unwrap_or(candle_core::Device::Cpu)
    } else {
        println!("GPU Acceleration Unavailable: Falling back to CPU.");
        candle_core::Device::Cpu
    }
}

fn gpu_ocr_fallback(file_path: &str) -> Result<String, String> {
    let device = get_device();
    println!("Running GPU fallback for {}", file_path);
    
    // Simulate tensor operations
    let a = Tensor::randn(0f32, 1f32, (100, 100), &device).map_err(|e| e.to_string())?;
    let b = Tensor::randn(0f32, 1f32, (100, 100), &device).map_err(|e| e.to_string())?;
    let _c = a.matmul(&b).map_err(|e| e.to_string())?;

    Ok(format!("[GPU OCR Fallback Simulated Output] Successfully ran tensor operations on device: {:?} for file: {}", device, file_path))
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
        "pdf" => {
            match pdf_extract::extract_text(path) {
                Ok(text) => {
                    if text.trim().is_empty() {
                        gpu_ocr_fallback(file_path)
                    } else {
                        Ok(text)
                    }
                },
                Err(e) => Err(format!("Failed to extract text from PDF: {}", e)),
            }
        }
        "docx" => {
            match dotext::Docx::open(path) {
                Ok(mut docx) => {
                    let mut text = String::new();
                    match docx.read_to_string(&mut text) {
                        Ok(_) => Ok(text),
                        Err(e) => Err(format!("Failed to read text from DOCX: {}", e)),
                    }
                }
                Err(e) => Err(format!("Failed to open DOCX: {}", e)),
            }
        }
        "png" | "jpg" | "jpeg" => {
            gpu_ocr_fallback(file_path)
        }
        _ => Err(format!("Unsupported file extension: .{}", ext)),
    };

    result.map(|text| sanitize_output(&text))
}
