# MarkItDown Native

This is a blazingly fast, pure Rust, local-first document parsing engine. It converts PDFs, Word Documents, HTML, and text files into LLM-readable Markdown entirely on your local machine—no Python environment or cloud APIs required.

## What's in this folder?
You will see two standalone executable files:
1. `markitdown-app.exe` (The visual drag-and-drop UI)
2. `mcp-server.exe` (The headless background server for your IDE)

---

## 1. Using the Desktop App (Manual Mode)
If you just want to quickly convert a file to Markdown to read or copy:
* Double-click `markitdown-app.exe`.
* Drag and drop any PDF, DOCX, or text file into the window.
* Click "Save as .md" if you want to save the physical output file to your machine.

---

## 2. Using the VS Code AI Agent (MCP Mode)
The real power of this architecture is letting your VS Code AI (like Roo Code or Cline) silently read your local documents in the background without cluttering your project with `.md` files.

**Setup Instructions:**
1. Move the `mcp-server.exe` file to a permanent, safe folder on your computer (e.g., `C:\Tools\MarkItDown\mcp-server.exe`).
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