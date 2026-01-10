const { execSync } = require('child_process');
const os = require('os');

/**
 * 激進模式：清理所有 Rust 工具鏈相關進程
 * ⚠️ 警告：這會終止所有 Rust 開發工具，包括 VSCode 的 rust-analyzer
 * 使用時機：當系統卡死，需要完全重置 Rust 開發環境
 */

console.log('💀 激進清理模式啟動...');
console.log('⚠️  這將終止所有 Rust 相關進程，包括 IDE 擴展！\n');

const AGGRESSIVE_TARGETS = [
  'build-script-build',
  'cargo',
  'cargo-clippy',
  'cargo-check',
  'rustc',
  'rust-analyzer',
  'rust-analyzer-proc-macro-srv',
  'rls',
  'dataset-app'
];

function cleanupWindows() {
  let totalKilled = 0;

  AGGRESSIVE_TARGETS.forEach(name => {
    try {
      const countCmd = `powershell -Command "Get-Process -Name '*${name}*' -ErrorAction SilentlyContinue | Measure-Object | Select-Object -ExpandProperty Count"`;
      const count = parseInt(execSync(countCmd, { encoding: 'utf-8' }).trim()) || 0;

      if (count > 0) {
        execSync(`powershell -Command "Get-Process -Name '*${name}*' -ErrorAction SilentlyContinue | Stop-Process -Force"`, { stdio: 'ignore' });
        totalKilled += count;
        console.log(`✅ 已終止 ${count} 個 ${name} 進程`);
      }
    } catch (err) {
      // 忽略錯誤
    }
  });

  return totalKilled;
}

function cleanupUnix() {
  let totalKilled = 0;

  AGGRESSIVE_TARGETS.forEach(name => {
    try {
      const countCmd = `pgrep -f "${name}" 2>/dev/null | wc -l`;
      const count = parseInt(execSync(countCmd, { encoding: 'utf-8' }).trim()) || 0;

      if (count > 0) {
        execSync(`pkill -9 -f "${name}" 2>/dev/null`, { stdio: 'ignore' });
        totalKilled += count;
        console.log(`✅ 已終止 ${count} 個 ${name} 進程`);
      }
    } catch (err) {
      // 忽略錯誤
    }
  });

  return totalKilled;
}

const platform = os.platform();
const totalKilled = platform === 'win32' ? cleanupWindows() : cleanupUnix();

if (totalKilled === 0) {
  console.log('\n✨ 沒有發現任何 Rust 進程');
} else {
  console.log(`\n💀 總共終止了 ${totalKilled} 個進程`);
  console.log(`💾 預估釋放記憶體: ~${(totalKilled * 100 / 1024).toFixed(2)} GB`);
  console.log('\n⚠️  請重新載入 VSCode 視窗以重啟 rust-analyzer');
}
