import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { FileText, Upload, AlertCircle, CheckCircle2, Download } from "lucide-react";

function App() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [output, setOutput] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [isSaved, setIsSaved] = useState(false);

  useEffect(() => {
    // Listen for file drops directly from Tauri
    const unlistenDrop = listen<{ paths: string[] }>("tauri://drop", (event) => {
      setIsDragging(false);
      if (event.payload.paths && event.payload.paths.length > 0) {
        const path = event.payload.paths[0];
        setFilePath(path);
        processFile(path);
      }
    });

    const unlistenDragEnter = listen("tauri://drag-enter", () => {
      setIsDragging(true);
    });

    const unlistenDragLeave = listen("tauri://drag-leave", () => {
      setIsDragging(false);
    });

    return () => {
      unlistenDrop.then(f => f());
      unlistenDragEnter.then(f => f());
      unlistenDragLeave.then(f => f());
    };
  }, []);

  const handleSelectFile = async () => {
    try {
      setError("");
      setOutput("");
      
      const selected = await open({
        multiple: false,
        filters: [{
          name: "Documents",
          extensions: ["txt", "md", "pdf", "docx", "png", "jpg", "jpeg"]
        }]
      });
      
      if (selected && typeof selected === "string") {
        setFilePath(selected);
        processFile(selected);
      }
    } catch (err) {
      console.error(err);
      setError("Failed to open file dialog");
    }
  };

  const processFile = async (path: string) => {
    setIsLoading(true);
    setError("");
    setOutput("");
    setIsSaved(false);
    
    try {
      const result: string = await invoke("convert_to_markdown", { filePath: path });
      setOutput(result);
    } catch (err) {
      console.error(err);
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSaveToDisk = () => {
    if (!output) return;
    
    const blob = new Blob([output], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    
    let filename = "document.md";
    if (filePath) {
      const originalName = filePath.split(/[\\/]/).pop() || "document";
      filename = originalName.replace(/\.[^/.]+$/, "") + ".md";
    }
    
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    
    setIsSaved(true);
    setTimeout(() => setIsSaved(false), 2000);
  };

  return (
    <div className="min-h-screen bg-stone-50 flex flex-col items-center justify-center p-8 text-stone-900 select-none">
      <div data-tauri-drag-region className="absolute top-0 left-0 right-0 h-10" />
      <div className="w-full max-w-3xl bg-white rounded-2xl border border-stone-200 p-8 z-10 relative">
        
        <div className="text-center mb-8">
          <h1 className="text-3xl font-medium tracking-tight">MarkIt<span className="font-bold">Native</span></h1>
          <p className="text-stone-500 mt-2 text-sm">Convert documents to Markdown instantly for AI processing</p>
        </div>

        <div 
          onClick={handleSelectFile}
          className={`w-full border-2 rounded-xl p-12 flex flex-col items-center justify-center cursor-pointer transition-all group
            ${isDragging 
              ? "bg-black text-white border-black scale-[1.02]" 
              : "bg-stone-50 text-stone-500 border-dashed border-stone-300 hover:border-black hover:bg-stone-100 hover:text-black"
            }`}
        >
          <div className={`mb-4 transition-transform ${isDragging ? "scale-110" : "group-hover:scale-110"}`}>
            <Upload className="w-8 h-8" strokeWidth={1.5} />
          </div>
          <p className="text-lg font-medium">
            {isDragging ? "Drop your file here" : "Drag & drop or click to select"}
          </p>
          <p className={`text-sm mt-1 ${isDragging ? "text-stone-300" : "text-stone-400"}`}>Supports .txt, .md, .pdf, .docx, images</p>
        </div>

        {error && (
          <div className="mt-6 p-4 bg-red-50 text-red-700 rounded-lg flex items-start gap-3 border border-red-100">
            <AlertCircle className="w-5 h-5 shrink-0 mt-0.5" />
            <p className="text-sm">{error}</p>
          </div>
        )}

        {(output || isLoading) && (
          <div className="mt-8 border border-stone-200 bg-white rounded-xl overflow-hidden">
            <div className="bg-stone-50 border-b border-stone-200 px-4 py-3 flex items-center gap-2">
              <FileText className="w-4 h-4 text-stone-500" strokeWidth={1.5} />
              <span className="text-sm font-medium text-stone-700 truncate flex-1">
                {filePath ? filePath.split(/[\\/]/).pop() : "Output"}
              </span>
              {isLoading && <span className="text-xs text-stone-500 font-medium animate-pulse">Processing...</span>}
              {!isLoading && output && <CheckCircle2 className="w-4 h-4 text-stone-900" />}
            </div>
            <div className="p-4 h-64 overflow-y-auto select-text">
              {isLoading ? (
                <div className="flex items-center justify-center h-full">
                  <div className="flex space-x-2 animate-pulse">
                    <div className="w-2.5 h-2.5 bg-stone-300 rounded-full"></div>
                    <div className="w-2.5 h-2.5 bg-stone-300 rounded-full"></div>
                    <div className="w-2.5 h-2.5 bg-stone-300 rounded-full"></div>
                  </div>
                </div>
              ) : (
                <pre className="text-sm text-stone-800 whitespace-pre-wrap font-mono">
                  {output}
                </pre>
              )}
            </div>
          </div>
        )}

        {output && !isLoading && (
          <div className="mt-4 flex justify-end">
            <button 
              onClick={handleSaveToDisk}
              disabled={isSaved}
              className={`flex items-center gap-2 px-4 py-2 border rounded-lg text-sm font-medium transition-colors
                ${isSaved 
                  ? "bg-black border-black text-white" 
                  : "bg-white border-stone-200 text-stone-900 hover:border-black"
                }`}
            >
              {isSaved ? (
                <>
                  <CheckCircle2 className="w-4 h-4" />
                  Saved!
                </>
              ) : (
                <>
                  <Download className="w-4 h-4" />
                  Save as .md
                </>
              )}
            </button>
          </div>
        )}

      </div>
    </div>
  );
}

export default App;
