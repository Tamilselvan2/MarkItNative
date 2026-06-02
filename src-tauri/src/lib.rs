pub mod parser;
pub mod vision_processor;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn convert_to_markdown(file_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        parser::parse_file(&file_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![convert_to_markdown])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use candle_core::{Device, Tensor};

    #[test]
    fn test_cuda_is_available_and_tensor_calc() -> anyhow::Result<()> {
        println!("Checking for CUDA availability...");
        assert!(candle_core::utils::cuda_is_available(), "CUDA is not available according to candle_core!");
        
        let device = Device::new_cuda(0)?;
        println!("✅ Connected to CUDA device: {:?}", device);
        
        let a = Tensor::new(&[1.0f32, 2.0, 3.0], &device)?;
        let b = Tensor::new(&[4.0f32, 5.0, 6.0], &device)?;
        
        let c = (&a * &b)?;
        
        let res = c.to_vec1::<f32>()?;
        assert_eq!(res, vec![4.0, 10.0, 18.0]);
        println!("✅ Tensor calculation on GPU successful! A * B = {:?}", res);
        
        Ok(())
    }
}
