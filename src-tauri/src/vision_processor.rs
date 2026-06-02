use anyhow::{Context, Result};
use candle_core::{Device, Tensor, DType};
use candle_transformers::{
    models::{moondream, quantized_moondream},
    quantized_var_builder::VarBuilder,
};
use hf_hub::{api::sync::Api, Repo, RepoType};

use pdfium_render::prelude::*;

pub fn get_optimal_device() -> candle_core::Result<Device> {
    if candle_core::utils::cuda_is_available() {
        Ok(Device::new_cuda(0)?)
    } else if candle_core::utils::metal_is_available() {
        Ok(Device::new_metal(0)?)
    } else {
        Ok(Device::Cpu)
    }
}

pub fn extract_text_from_image(path: &str) -> Result<String, String> {
    extract_text_from_image_impl(path).map_err(|e| format!("Vision processor error: {:?}", e))
}

fn extract_text_from_image_impl(path: &str) -> Result<String> {
    let device = get_optimal_device().unwrap_or(Device::Cpu);
    
    // 1. Initialize API and Fetch Quantized Moondream Files
    let api = Api::new().context("API init error")?;
    let repo = api.repo(Repo::with_revision(
        "santiagomed/candle-moondream".to_string(),
        RepoType::Model,
        "main".to_string(),
    ));
    
    // Fetch 4-bit Quantized GGUF Weights
    let model_file = repo.get("model-q4_0.gguf").context("Model fetch error")?;
    let tokenizer_file = repo.get("tokenizer.json").context("Tokenizer fetch error")?;

    // 2. Load and Preprocess Image / PDF
    let img = if path.to_lowercase().ends_with(".pdf") {
        let pdfium = Pdfium::new(Pdfium::bind_to_system_library().context("Could not find pdfium.dll in path")?);
        let document = pdfium.load_pdf_from_file(path, None).context("Failed to load PDF file")?;
        let page = document.pages().get(0).context("PDF has no pages")?;
        
        // Render at high resolution to ensure crisp text for OCR
        let config = PdfRenderConfig::new().set_target_width(2000);
        let rendered_image = page.render_with_config(&config)
            .context("Failed to render PDF page")?
            .as_image();
        rendered_image
    } else {
        image::ImageReader::open(path)?.decode()?
    };
    let img = img.resize_to_fill(378, 378, image::imageops::FilterType::Triangle);
    let img = img.to_rgb8();
    let data = img.into_raw();
    
    let data_tensor = Tensor::from_vec(data, (378, 378, 3), &Device::Cpu)?.permute((2, 0, 1))?;
    let mean = Tensor::new(&[0.5f32, 0.5, 0.5], &Device::Cpu)?.reshape((3, 1, 1))?;
    let std = Tensor::new(&[0.5f32, 0.5, 0.5], &Device::Cpu)?.reshape((3, 1, 1))?;
    
    // Quantized Moondream strictly requires F32 processing for image operations
    let mut tensor = data_tensor.to_dtype(DType::F32)?;
    tensor = (tensor / 255.0)?;
    tensor = tensor.broadcast_sub(&mean)?;
    tensor = tensor.broadcast_div(&std)?;
    tensor = tensor.to_device(&device)?;
    let tensor = tensor.unsqueeze(0)?;

    // 3. Load Quantized Model and Config
    let config = moondream::Config::v2();
    let vb = VarBuilder::from_gguf(&model_file, &device)?;
    let mut model = quantized_moondream::Model::new(&config, vb)?;
    
    let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_file)
        .map_err(|e| anyhow::anyhow!("Tokenizer load error: {}", e))?;

    // 4. Encode Image
    let image_embeds = tensor.apply(model.vision_encoder())?;

    // 5. Prepare Text Prompt
    let prompt_text = "Extract the text, tables, and layout of this document into clean Markdown.";
    let prompt = format!("\n\nQuestion: {0}\n\nAnswer:", prompt_text);
    let tokens = tokenizer.encode(prompt, true).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut tokens = tokens.get_ids().to_vec();

    let special_token = match tokenizer.get_vocab(true).get("<|endoftext|>") {
        Some(token) => *token,
        None => anyhow::bail!("cannot find the special token"),
    };

    // 6. Generation Loop
    let mut generated_tokens = Vec::new();

    for index in 0..2048 {
        let context_size = if index > 0 { 1 } else { tokens.len() };
        let ctxt = &tokens[tokens.len().saturating_sub(context_size)..];
        let input = Tensor::new(ctxt, &device)?.unsqueeze(0)?;
        
        let logits = if index > 0 {
            model.text_model.forward(&input)?
        } else {
            let bos_tensor = Tensor::new(&[special_token], &device)?.unsqueeze(0)?;
            model.text_model.forward_with_img(&bos_tensor, &input, &image_embeds)?
        };
        
        // Ensure logits are back to F32 for sampling
        let logits = logits.squeeze(0)?.to_dtype(DType::F32)?;
        // Pure greedy decoding to prevent hallucinations
        let next_token = logits.argmax(candle_core::D::Minus1)?.to_scalar::<u32>()?;
        
        tokens.push(next_token);
        generated_tokens.push(next_token);
        
        if next_token == special_token || tokens.ends_with(&[27, 10619, 29]) {
            break;
        }
    }

    // 7. Decode output
    let decoded = tokenizer.decode(&generated_tokens, true).map_err(|e| anyhow::anyhow!("Token decode error: {}", e))?;
    
    let final_markdown = decoded.trim().to_string();
    println!("Moondream Extracted Markdown:\n{}", final_markdown);
    
    Ok(final_markdown)
}
