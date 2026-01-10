const { execSync } = require('child_process');
const os = require('os');

/**
 * 跨平台清理 Rust 編譯殭屍進程
 * 支援 Windows, macOS, Linux
 */

const ZOMBIE_PROCESSES = [
  'build-script-build',
  'cargo',
  'rust-analyzer',
  'dataset-app'
];

function cleanupWindows() {
  console.log('🧟 正在清理 Windows 殭屍進程...');
  let totalKilled = 0;

  ZOMBIE_PROCESSES.forEach(name => {
    try {
      // 使用 Get-Process 配合通配符來匹配所有變體（包括帶數字後綴的）
      const countCmd = `powershell -Command "Get-Process -Name '*${name}*' -ErrorAction SilentlyContinue | Measure-Object | Select-Object -ExpandProperty Count"`;
      const count = parseInt(execSync(countCmd, { encoding: 'utf-8' }).trim()) || 0;

      if (count > 0) {
        // 強制終止所有匹配的進程
        execSync(`powershell -Command "Get-Process -Name '*${name}*' -ErrorAction SilentlyContinue | Stop-Process -Force"`, { stdio: 'ignore' });
        totalKilled += count;
        console.log(`✅ 已終止 ${count} 個 ${name} 進程`);
      }
    } catch (err) {
      // 忽略找不到進程的錯誤
    }
  });

  if (totalKilled === 0) {
    console.log('✨ 沒有發現殭屍進程，系統很乾淨！');
  } else {
    console.log(`\n🎉 總共清理了 ${totalKilled} 個殭屍進程`);
    const memFreed = (totalKilled * 100 / 1024).toFixed(2);
    console.log(`💾 預估釋放記憶體: ~${memFreed} GB`);
  }
}

function cleanupUnix() {
  console.log('🧟 正在清理 Unix/Linux 殭屍進程...');
  let totalKilled = 0;

  ZOMBIE_PROCESSES.forEach(name => {
    try {
      // 使用 pgrep 計數
      const countCmd = `pgrep -f "${name}" 2>/dev/null | wc -l`;
      const count = parseInt(execSync(countCmd, { encoding: 'utf-8' }).trim()) || 0;

      if (count > 0) {
        // 使用 pkill -9 強制終止
        execSync(`pkill -9 -f "${name}" 2>/dev/null`, { stdio: 'ignore' });
        totalKilled += count;
        console.log(`✅ 已終止 ${count} 個 ${name} 進程`);
      }
    } catch (err) {
      // 忽略錯誤
    }
  });

  if (totalKilled === 0) {
    console.log('✨ 沒有發現殭屍進程，系統很乾淨！');
  } else {
    console.log(`\n🎉 總共清理了 ${totalKilled} 個殭屍進程`);
    const memFreed = (totalKilled * 100 / 1024).toFixed(2);
    console.log(`💾 預估釋放記憶體: ~${memFreed} GB`);
  }
}

function main() {
  const platform = os.platform();

  console.log(`🔍 偵測到平台: ${platform}\n`);

  switch (platform) {
    case 'win32':
      cleanupWindows();
      break;
    case 'darwin':
    case 'linux':
      cleanupUnix();
      break;
    default:
      console.error(`❌ 不支援的平台: ${platform}`);
      process.exit(1);
  }
}

main();
