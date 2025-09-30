import { useEffect, useRef, useState, useCallback } from "react";
import Swal from "sweetalert2";
import "./Repl.css";


type CodeMirrorEditor = {
  getValue: () => string;
  setValue: (value: string) => void;
  toTextArea?: () => void;
  refresh?: () => void;
};

export default function WppRepl() {
  const editorRef = useRef<HTMLDivElement>(null);
  const [editor, setEditor] = useState<CodeMirrorEditor | null>(null);
  const [editorReady, setEditorReady] = useState(false);
  const [files, setFiles] = useState<Record<string, string>>({ "main.wpp": "" });
  const [activeFile, setActiveFile] = useState("main.wpp");
  const [output, setOutput] = useState("Ready.");
  const [dragging, setDragging] = useState(false);
  const [outputWidth, setOutputWidth] = useState(30); // %
  const [mode, setMode] = useState<"interp" | "jit">("interp");

  useEffect(() => {
    if (!editorRef.current || !editorReady || editor) return;

    const CodeMirror = (window as any).CodeMirror;
    if (typeof CodeMirror !== "function") return;

    const instance = CodeMirror(editorRef.current, {
      mode: "wpp",
      lineNumbers: true,
      theme: "material",
    });

    instance.setValue(files[activeFile]);
    setEditor(instance);
    setTimeout(() => instance.refresh?.(), 0);
    return () => instance.toTextArea?.();
  }, [editorReady, editor, activeFile, files]);

  useEffect(() => {
  const interval = setInterval(() => {
    if ((window as any).CodeMirror) {
      setEditorReady(true);
      clearInterval(interval);
    }
  }, 100);
  return () => clearInterval(interval);
}, []);


  const switchToFile = (name: string) => {
    if (!editor) return;
    setFiles((prev) => {
      const updated = { ...prev, [activeFile]: editor.getValue() };
      editor.setValue(updated[name] || "");
      return updated;
    });
    setActiveFile(name);
  };

  const addFile = async () => {
  const { value: name } = await Swal.fire({
    title: "Enter file name",
    input: "text",
    inputLabel: "The file will end with .wpp",
    inputPlaceholder: "example",
    showCancelButton: true,
    confirmButtonText: "Add",
    inputValidator: (value) => {
      if (!value) return "Please enter a name!";
      const safe = value.endsWith(".wpp") ? value : `${value}.wpp`;
      if (files[safe]) return "File already exists!";
      return null;
    },
  });

  if (!name) return;

  const safeName = name.endsWith(".wpp") ? name : `${name}.wpp`;
  setFiles((prev) => ({ ...prev, [safeName]: "" }));
  setActiveFile(safeName);
  setTimeout(() => editor?.setValue(""), 0);
};

const removeFile = async (name: string) => {
  const result = await Swal.fire({
    title: `Remove ${name}?`,
    text: "This action cannot be undone.",
    icon: "warning",
    showCancelButton: true,
    confirmButtonColor: "#e11d48", // red
    confirmButtonText: "Yes, delete it!",
  });

  if (result.isConfirmed) {
    const updated = { ...files };
    delete updated[name];
    const newActive = Object.keys(updated)[0] || "";
    setFiles(updated);
    setActiveFile(newActive);
    editor?.setValue(updated[newActive] || "");
  }
};


  const runCode = async () => {
  if (!editor) return;
  const updatedFiles = { ...files, [activeFile]: editor.getValue() };
  const { [activeFile]: code } = updatedFiles;

  setOutput("Running...");
  try {
    const res = await fetch("https://wppreplbackend.onrender.com/api/run/wpp", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ code, mode }),
});

    const data = await res.json();
    setOutput(data.output || data.error || "No output.");
  } catch (e) {
    setOutput("Error: " + (e instanceof Error ? e.message : String(e)));
  }
  setFiles(updatedFiles);
};


  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!dragging) return;
    const percent = ((window.innerWidth - e.clientX) / window.innerWidth) * 100;
    setOutputWidth(Math.max(20, Math.min(50, percent)));
  }, [dragging]);

  useEffect(() => {
    window.addEventListener("mouseup", () => setDragging(false));
    window.addEventListener("mousemove", handleMouseMove);
    return () => {
      window.removeEventListener("mouseup", () => setDragging(false));
      window.removeEventListener("mousemove", handleMouseMove);
    };
  }, [handleMouseMove]);

  const handleMouseDown = () => setDragging(true);

  return (
    <>
      
      <div className="repl-container">
        {/* Toolbar */}
        <div className="repl-toolbar-wpp">
  <button onClick={runCode}>▶</button>
  <button
    className="mode-toggle"
    onClick={() => setMode((prev) => (prev === "interp" ? "jit" : "interp"))}
    style={{
      background: mode === "jit" ? "#e11d48" : "#0ea5e9",
    }}
  >
    {mode === "jit" ? "JIT Mode" : "Interpreter Mode"}
  </button>
</div>




        {/* Editor + Tabs */}
        <div className="repl-editor-panel">
          <div className="repl-tabs">
            {Object.keys(files).map((file) => (
              <div
                key={file}
                onClick={() => switchToFile(file)}
                className={`repl-tab ${file === activeFile ? "active" : ""}`}
              >
                {file}
                <span
                  className="repl-tab-close"
                  onClick={(e) => {
                    e.stopPropagation();
                    removeFile(file);
                  }}
                >
                  ×
                </span>
              </div>
            ))}
            <button onClick={addFile} className="repl-add-tab">
              +
            </button>
          </div>
          <div ref={editorRef} className="repl-editor" />
        </div>

        <div onMouseDown={handleMouseDown} className="repl-divider" />

        {/* Output */}
        <div className="repl-output" style={{ width: `${outputWidth}%` }}>
          <div className="repl-output-header">OUTPUT</div>
          <pre>{output}</pre>
        </div>
      </div>
    </>
  );
}
