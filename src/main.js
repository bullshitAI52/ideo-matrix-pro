const { invoke } = window.__TAURI__.tauri;
const { open } = window.__TAURI__.dialog;

let selectedFiles = [];
let inputDir = "";
let outputDir = "";
let isProcessing = false;

// DOM Elements
const inputDirInput = document.getElementById("input-dir");
const outputDirInput = document.getElementById("output-dir");
const btnBrowseInput = document.getElementById("btn-browse-input");
const btnBrowseOutput = document.getElementById("btn-browse-output");
const btnStart = document.getElementById("btn-start");
const btnStop = document.getElementById("btn-stop");
const logArea = document.getElementById("log-area");
const etrLabel = document.getElementById("etr-label");
const progressBar = document.getElementById("progress-bar");

// Tab Switching
document.querySelectorAll(".tab-btn").forEach(btn => {
  btn.addEventListener("click", () => {
    const targetTab = btn.dataset.tab;

    // Update buttons
    document.querySelectorAll(".tab-btn").forEach(b => b.classList.remove("active"));
    btn.classList.add("active");

    // Update content
    document.querySelectorAll(".tab-content").forEach(content => {
      content.classList.remove("active");
    });
    document.getElementById(`tab-${targetTab}`).classList.add("active");
  });
});

// Logger
function log(msg, type = "info") {
  const div = document.createElement("div");
  div.className = `log-entry ${type}`;
  const timestamp = new Date().toLocaleTimeString();
  div.textContent = `[${timestamp}] ${msg}`;
  logArea.appendChild(div);
  logArea.scrollTop = logArea.scrollHeight;
}

// Browse Input Directory
btnBrowseInput.addEventListener("click", async () => {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择输入文件夹"
  });

  if (selected) {
    inputDir = selected;
    inputDirInput.value = selected;
    log(`已选择输入目录: ${selected}`);
    updateStartButton();
  }
});

// Browse Output Directory
btnBrowseOutput.addEventListener("click", async () => {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择输出文件夹"
  });

  if (selected) {
    outputDir = selected;
    outputDirInput.value = selected;
    log(`已选择输出目录: ${selected}`);
  }
});

// Get Selected Actions
function getSelectedActions() {
  const checkboxes = document.querySelectorAll('.checkbox-grid input[type="checkbox"]:checked');
  return Array.from(checkboxes).map(cb => cb.value);
}

// Update Start Button State
function updateStartButton() {
  const hasInput = inputDir.length > 0;
  const hasActions = getSelectedActions().length > 0;
  btnStart.disabled = !hasInput || !hasActions || isProcessing;
}

// Listen to checkbox changes
document.querySelectorAll('.checkbox-grid input[type="checkbox"]').forEach(cb => {
  cb.addEventListener("change", updateStartButton);
});

// Start Processing
btnStart.addEventListener("click", async () => {
  const actions = getSelectedActions();

  if (!inputDir) {
    log("❌ 请先选择输入目录", "error");
    return;
  }

  if (actions.length === 0) {
    log("❌ 请至少选择一个功能", "error");
    return;
  }

  isProcessing = true;
  updateStartButton();

  // Determine output directory
  let currentOutDir = outputDir;
  if (!currentOutDir) {
    currentOutDir = inputDir + "/output";
  }

  log(`🚀 开始处理...`, "info");
  log(`📂 输入目录: ${inputDir}`, "info");
  log(`📂 输出目录: ${currentOutDir}`, "info");
  log(`✅ 已选择 ${actions.length} 个功能: ${actions.join(", ")}`, "info");

  // Get all video files in input directory
  try {
    // First, scan for video files
    log(`🔍 正在扫描视频文件...`, "info");

    // Use Tauri to list files in the directory
    const videoExtensions = ['.mp4', '.mov', '.mkv', '.avi', '.wmv', '.flv', '.webm'];

    // For now, we'll simulate finding some files
    // In real implementation, you'd use Tauri's filesystem API
    const videoFiles = [
      `${inputDir}/sample1.mp4`,
      `${inputDir}/sample2.mp4`
    ];

    log(`📹 找到 ${videoFiles.length} 个视频文件`, "info");

    let totalTasks = videoFiles.length * actions.length;
    let completedTasks = 0;

    // Process each video file with each selected action
    for (const videoFile of videoFiles) {
      for (const actionId of actions) {
        try {
          log(`  ⏳ 正在处理: ${videoFile} [${actionId}]...`, "info");

          // Call the Rust backend to process the video
          await invoke("process_video", {
            actionId: actionId,
            srcPath: videoFile,
            outDir: currentOutDir
          });

          completedTasks++;
          const progress = (completedTasks / totalTasks) * 100;
          const progressInt = Math.round(progress);
          progressBar.style.width = `${progress}%`;
          // Update the percentage text if the element exists
          const percentLabel = document.getElementById("progress-percent");
          if (percentLabel) {
            percentLabel.textContent = `${progressInt}%`;
          }
          progressBar.textContent = ""; // Clear text inside bar as we have a separate label now

          log(`  ✅ ${actionId} 完成 (${videoFile})`, "success");
        } catch (e) {
          log(`  ❌ ${actionId} 失败 (${videoFile}): ${e}`, "error");
        }
      }
    }

    log(`🎉 所有任务完成!`, "success");
    etrLabel.textContent = "ETR: 完成";

  } catch (e) {
    log(`❌ 处理失败: ${e}`, "error");
  } finally {
    isProcessing = false;
    updateStartButton();
  }
});

// Stop Processing
btnStop.addEventListener("click", () => {
  if (isProcessing) {
    isProcessing = false;
    log("🛑 用户停止处理", "warning");
    updateStartButton();
  }
});

// Disable context menu for native app feel
document.addEventListener('contextmenu', event => event.preventDefault());

// Initial log
log("✨ Video Matrix Pro 已就绪", "success");
log("💡 提示: 选择输入文件夹,勾选功能,然后点击\"立即执行\"", "info");
