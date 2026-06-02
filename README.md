# MarkItNative

MarkItNative is a blazingly fast, pure Rust, local-first document parsing engine wrapped in a modern Tauri interface. It converts PDFs, Word Documents, HTML, Text, and Images (OCR) into LLM-readable Markdown entirely on your local machine—no Python environment, cloud APIs, or external dependencies required.

## Features
- **Desktop Application:** A hyper-minimalist, drag-and-drop UI for manually converting documents to Markdown.
- **Headless MCP Server:** A background server (`mcp-server.exe`) that allows AI coding agents (like Roo Code, Cline, etc.) to natively read and process your local documents directly inside your IDE.
- **Local-First Processing:** Everything happens on your machine using efficient Rust libraries (`candle-core`, `pdf-extract`, `dotext`, and `pdfium-render`).
- **Native Vision-Language OCR:** Extracts text, tables, and layouts from images and scanned PDFs using the 4-bit quantized **Moondream2** VLM running directly on your GPU.

---

## Installation (Pre-built Binaries)

You can download the pre-compiled executables from the [Releases](../../releases) page. The release contains two files:
1. `markitnative.exe` - The main desktop application.
2. `mcp-server.exe` - The headless Model Context Protocol (MCP) server.

---

## Building from Source

If you prefer to build MarkItNative from source, ensure you have [Rust](https://www.rust-lang.org/), [Node.js](https://nodejs.org/), and the [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) installed.

```bash
# Clone the repository
git clone https://github.com/Tamilselvan2/MarkItNative.git
cd MarkItNative

# Install frontend dependencies
npm install

# Build the desktop app and MCP server
npm run tauri build
```
The compiled binaries will be located in `src-tauri/target/release/`.

---

## Vision-Language OCR Pipeline (Image & Scanned PDF Flow)

MarkItNative natively supports OCR and structural extraction for Images (PNG, JPG) and scanned PDFs using a hardware-accelerated Vision-Language Model.

**How it works:**
1. **Detection:** When a file is processed, `parser.rs` attempts standard text extraction. If an image is passed, or if a `.pdf` yields zero text (indicating a scanned document), it triggers the `gpu_ocr_fallback()`.
2. **PDF Rasterization:** For PDFs, the system binds to `pdfium.dll` dynamically, rendering the PDF page to a crisp 2000px image buffer in memory.
3. **Moondream2 Processing:** The image buffer is fed into the `santiagomed/candle-moondream` 4-bit Quantized GGUF model via Hugging Face Hub. 
4. **Precision Decoding:** The model uses strict mathematical greedy decoding (`.argmax()`) to guarantee exact text extraction without LLM hallucinations.

*(Note: Running the OCR pipeline requires downloading the `pdfium.dll` binary into your execution directory).*

---

## Using the VS Code AI Agent (MCP Mode)

The real power of this architecture is letting your VS Code AI silently read your local documents in the background without cluttering your project with temporary `.md` files.

**Setup Instructions:**
1. Download or build `mcp-server.exe` and move it to a permanent, safe folder on your computer (e.g., `C:\Tools\MarkItNative\mcp-server.exe`).
2. Open VS Code.
3. Press **Ctrl + Shift + P** to open the Command Palette.
4. Search for your AI's MCP settings (e.g., `Roo Code: MCP Servers` or `Cline: MCP Servers`).
5. Add the following block to your `cline_mcp_settings.json` file. 

*(CRITICAL: You must change the `command` path below to wherever you saved the `mcp-server.exe` file on your specific computer. Use double backslashes `\\` for Windows paths!)*

```json
{
  "mcpServers": {
    "local-rust-parser": {
      "command": "C:\\Your\\Path\\Here\\mcp-server.exe",
      "args": [],
      "disabled": false,
      "alwaysAllow": [
        "read_and_convert_document"
      ]
    }
  }
}
```
