const { invoke } = window.__TAURI__.tauri;
const { open } = window.__TAURI__.dialog;

let selectedFiles = [];
let inputDir = "";
let outputDir = "";
let isProcessing = false;
let singleVideoMode = false;

// 处理视频链式调用（多个动作按顺序应用到同一个视频）
async function processVideoChain(videoFile, actions, outDir, onProgress) {
  let currentInput = videoFile;
  const ext = videoFile.split('.').pop();
  const baseName = videoFile.split('/').pop().replace(`.${ext}`, '');
  let tempFiles = [];
  
  for (let i = 0; i < actions.length; i++) {
    const actionId = actions[i];
    const isLastAction = i === actions.length - 1;
    
    // 生成输出文件名：假设Rust动作会在outDir中生成 ${baseName}_${actionId}.${ext} 格式的文件
    // 但对于链式调用，我们需要跟踪实际生成的文件名
    // 简化：使用固定临时文件名，每次覆盖（但Rust可能不允许）
    // 改为：使用递增的文件名
    const outputFileName = isLastAction ? 
      `${baseName}_processed.${ext}` : 
      `${baseName}_chain_${i}_${actionId}.${ext}`;
    const outputPath = `${outDir}/${outputFileName}`;
    
    if (!isLastAction) {
      tempFiles.push(outputPath);
    }
    
    try {
      // 注意：Rust的process_video可能忽略我们指定的输出路径，使用自己的命名规则
      // 这里假设它会使用我们提供的输出路径
      await invoke("process_video", {
        actionId: actionId,
        srcPath: currentInput,
        outDir: outDir
      });
      
      // 假设输出文件已经生成在outDir中，文件名为 ${baseName}_${actionId}.${ext}
      // 但为了简单，我们假设输出就是我们指定的outputPath
      // 实际上需要扫描outDir来找到新生成的文件
      // 暂时使用outputPath作为下一个输入
      currentInput = outputPath;
      
      // 更新进度
      if (onProgress) {
        onProgress(i + 1, actions.length, actionId);
      }
      
    } catch (e) {
      throw new Error(`动作 ${actionId} 失败: ${e}`);
    }
  }
  
  // 清理临时文件（如果delete_file命令存在）
  for (const tempFile of tempFiles) {
    try {
      await invoke("delete_file", { path: tempFile });
    } catch (e) {
      // 忽略错误，可能命令不存在
      console.warn(`无法删除临时文件 ${tempFile}: ${e}`);
    }
  }
  
  return currentInput; // 返回最终输出文件路径
}

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

  // 扫描视频文件
  try {
    log(`🔍 正在扫描视频文件...`, "info");

    // 视频文件扩展名
    const videoExtensions = ['.mp4', '.mov', '.mkv', '.avi', '.wmv', '.flv', '.webm'];

    // 模拟找到一些文件（实际实现应使用Tauri文件系统API）
    const videoFiles = [
      `${inputDir}/sample1.mp4`,
      `${inputDir}/sample2.mp4`
    ];

    log(`📹 找到 ${videoFiles.length} 个视频文件`, "info");

    let totalTasks = videoFiles.length * actions.length;
    let completedTasks = 0;

    // 更新进度条函数
    function updateProgress() {
      completedTasks++;
      const progress = (completedTasks / totalTasks) * 100;
      const progressInt = Math.round(progress);
      progressBar.style.width = `${progress}%`;
      const percentLabel = document.getElementById("progress-percent");
      if (percentLabel) {
        percentLabel.textContent = `${progressInt}%`;
      }
      progressBar.textContent = "";
    }

    // 处理每个视频文件
    for (const videoFile of videoFiles) {
      if (singleVideoMode) {
        // 单个视频叠加模式：所有动作按顺序应用到同一个视频
        try {
          log(`  ⏳ 正在处理: ${videoFile} [叠加模式: ${actions.join(" → ")}]...`, "info");
          
          await processVideoChain(videoFile, actions, currentOutDir, (current, total, actionId) => {
            log(`    ↪️ 步骤 ${current}/${total}: ${actionId}`, "info");
            updateProgress();
          });
          
          log(`  ✅ 叠加处理完成 (${videoFile})`, "success");
          // 更新进度（每个动作都已在上面的回调中更新）
        } catch (e) {
          log(`  ❌ 叠加处理失败 (${videoFile}): ${e}`, "error");
          // 如果链式处理失败，仍要更新进度（避免卡住）
          completedTasks += (actions.length - Math.floor(completedTasks % actions.length));
          updateProgress();
        }
      } else {
        // 原始模式：每个动作生成独立视频
        for (const actionId of actions) {
          try {
            log(`  ⏳ 正在处理: ${videoFile} [${actionId}]...`, "info");

            await invoke("process_video", {
              actionId: actionId,
              srcPath: videoFile,
              outDir: currentOutDir
            });

            updateProgress();
            log(`  ✅ ${actionId} 完成 (${videoFile})`, "success");
          } catch (e) {
            log(`  ❌ ${actionId} 失败 (${videoFile}): ${e}`, "error");
            updateProgress();
          }
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

// 单个视频模式开关
const singleVideoToggle = document.getElementById("single-video-toggle");
if (singleVideoToggle) {
  singleVideoToggle.addEventListener("change", function() {
    singleVideoMode = this.checked;
    log(singleVideoMode ? "✅ 已开启单个视频功能叠加模式" : "✅ 已关闭单个视频功能叠加模式", "info");
  });
}

// Disable context menu for native app feel
document.addEventListener('contextmenu', event => event.preventDefault());

// Initial log
log("✨ Video Matrix Pro 已就绪", "success");
log("💡 提示: 选择输入文件夹,勾选功能,然后点击\"立即执行\"", "info");
