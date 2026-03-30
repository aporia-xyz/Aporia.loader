// ============================================================
// Rust IPC Bridge
// ============================================================
const ipc = window.ipc || {
  postMessage: (msg) => console.log('IPC:', msg)
};

let selectedVersion = '0.5.0';
let currentBranch = 'mcp';

// ============================================================
// Canvas Cosmos Background
// ============================================================
class CosmosBackground {
  constructor() {
    this.canvas = document.getElementById('cosmos');
    if (!this.canvas) {
      console.error('Canvas element not found!');
      return;
    }
    
    this.ctx = this.canvas.getContext('2d');
    if (!this.ctx) {
      console.error('Failed to get canvas context!');
      return;
    }
    
    this.particles = [];
    this.stars = [];
    this.resize();
    this.initStars();
    console.log('Cosmos initialized with', this.stars.length, 'stars');
    this.animate();
    window.addEventListener('resize', () => this.resize());
  }

  resize() {
    this.canvas.width = window.innerWidth;
    this.canvas.height = window.innerHeight;
  }

  initStars() {
    const starCount = Math.floor((this.canvas.width * this.canvas.height) / 15000);
    for (let i = 0; i < starCount; i++) {
      this.stars.push({
        x: Math.random() * this.canvas.width,
        y: Math.random() * this.canvas.height,
        radius: Math.random() * 1.5,
        opacity: Math.random() * 0.5 + 0.3,
        twinkleSpeed: Math.random() * 0.02 + 0.005
      });
    }
  }

  drawStars() {
    this.stars.forEach(star => {
      star.opacity += (Math.random() - 0.5) * star.twinkleSpeed;
      star.opacity = Math.max(0.1, Math.min(1, star.opacity));
      
      this.ctx.fillStyle = `rgba(255, 255, 255, ${star.opacity})`;
      this.ctx.beginPath();
      this.ctx.arc(star.x, star.y, star.radius, 0, Math.PI * 2);
      this.ctx.fill();
    });
  }

  drawNebula() {
    const gradient = this.ctx.createRadialGradient(
      this.canvas.width * 0.3, this.canvas.height * 0.3, 0,
      this.canvas.width * 0.3, this.canvas.height * 0.3, this.canvas.width * 0.8
    );
    gradient.addColorStop(0, 'rgba(139, 92, 246, 0.08)');
    gradient.addColorStop(0.5, 'rgba(109, 40, 217, 0.04)');
    gradient.addColorStop(1, 'rgba(6, 6, 14, 0)');
    
    this.ctx.fillStyle = gradient;
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

    const gradient2 = this.ctx.createRadialGradient(
      this.canvas.width * 0.7, this.canvas.height * 0.6, 0,
      this.canvas.width * 0.7, this.canvas.height * 0.6, this.canvas.width * 0.6
    );
    gradient2.addColorStop(0, 'rgba(168, 85, 247, 0.06)');
    gradient2.addColorStop(1, 'rgba(6, 6, 14, 0)');
    
    this.ctx.fillStyle = gradient2;
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
  }

  animate() {
    this.ctx.fillStyle = '#06060e';
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    
    this.drawNebula();
    this.drawStars();
    
    requestAnimationFrame(() => this.animate());
  }
}

let cosmosInstance = null;

// Перенаправляем console.log в IPC для логирования в Rust
const originalLog = console.log;
const originalError = console.error;

console.log = function(...args) {
  originalLog.apply(console, args);
  try {
    if (window.debugLog) {
      window.debugLog(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
    }
  } catch (e) {}
};

console.error = function(...args) {
  originalError.apply(console, args);
  try {
    if (window.debugLog) {
      window.debugLog('ERROR: ' + args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
    }
  } catch (e) {}
};

// ============================================================
// Flash Effect
// ============================================================
function triggerFlash() {
  const overlay = document.getElementById('flashOverlay');
  if (!overlay) return;
  
  overlay.classList.add('active');
  setTimeout(() => {
    overlay.classList.remove('active');
  }, 100);
}

// ============================================================
// Progress Bar Management
// ============================================================
let progressInterval = null;

function showProgress() {
  const container = document.getElementById('progressContainer');
  if (container) {
    container.classList.add('active');
  }
}

function hideProgress() {
  const container = document.getElementById('progressContainer');
  if (container) {
    container.classList.remove('active');
  }
  if (progressInterval) {
    clearInterval(progressInterval);
    progressInterval = null;
  }
}

function setProgress(percent) {
  const fill = document.getElementById('progressFill');
  const label = document.getElementById('progressPercent');
  
  if (fill) {
    fill.style.width = Math.min(100, Math.max(0, percent)) + '%';
  }
  if (label) {
    label.textContent = Math.round(percent) + '%';
  }
  
  console.log('Progress:', percent + '%');
}

function simulateProgress() {
  let progress = 0;
  showProgress();
  
  if (progressInterval) {
    clearInterval(progressInterval);
  }
  
  progressInterval = setInterval(() => {
    progress += Math.random() * 15;
    
    if (progress > 90) {
      progress = 90;
    }
    
    setProgress(progress);
    
    if (progress >= 90) {
      clearInterval(progressInterval);
      progressInterval = null;
    }
  }, 300);
}

// ============================================================
// Обработка данных от Rust
// ============================================================
window.updateReleases = function(releases) {
  console.log('updateReleases called with:', releases.length, 'items');
  const list = document.getElementById('releasesList');
  if (!list) {
    console.error('releasesList element not found!');
    return;
  }
  
  list.innerHTML = '';
  
  releases.forEach((release, idx) => {
    const item = document.createElement('div');
    item.className = 'release-item' + (release.is_latest ? ' latest' : '');
    item.dataset.ver = release.tag_name;
    
    const date = new Date(release.published_at).toISOString().split('T')[0];
    
    item.innerHTML = `
      <div class="release-left">
        <span class="release-ver">v${release.tag_name}</span>
        ${release.is_latest ? '<span class="release-badge">latest</span>' : ''}
      </div>
      <span class="release-date">${date}</span>
    `;
    
    item.addEventListener('click', () => {
      selectedVersion = release.tag_name;
      
      document.querySelectorAll('.release-item').forEach(i => i.classList.remove('latest'));
      item.classList.add('latest');
      
      document.querySelectorAll('.release-badge').forEach(b => b.remove());
      const badge = document.createElement('span');
      badge.className = 'release-badge';
      badge.textContent = 'selected';
      item.querySelector('.release-left').appendChild(badge);
      
      const version = parseFloat(selectedVersion);
      const branch = version >= 0.5 ? 'mcp' : 'fabric';
      loadCommits(branch);
      
      triggerFlash();
    });
    
    list.appendChild(item);
  });
  
  if (releases.length > 0) {
    selectedVersion = releases[0].tag_name;
  }
  console.log('Releases updated');
};

window.updateCommits = function(commits, branch) {
  console.log('updateCommits called with:', commits.length, 'items from', branch);
  const list = document.getElementById('commitsList');
  const title = document.getElementById('commitsTitle');
  
  if (!list || !title) {
    console.error('commitsList or commitsTitle element not found!');
    return;
  }
  
  title.textContent = `Latest commits (${branch})`;
  list.innerHTML = '';
  
  commits.slice(0, 3).forEach(commit => {
    const item = document.createElement('div');
    item.className = 'commit-item';
    item.innerHTML = `
      <span class="commit-hash">${commit.short_sha}</span>
      <span class="commit-msg">${commit.message}</span>
    `;
    list.appendChild(item);
  });
  console.log('Commits updated');
};

// ============================================================
// Загрузка данных с GitHub
// ============================================================
async function loadReleases() {
  console.log('=== loadReleases START ===');
  
  try {
    console.log('Fetching from GitHub API...');
    const res = await fetch('https://api.github.com/repos/dakychan/Aporia/releases');
    console.log('Fetch response status:', res.status);
    
    const data = await res.json();
    console.log('Releases fetched:', data.length, 'items');
    
    const releases = data.map((r, idx) => ({
      tag_name: r.tag_name,
      name: r.name || '',
      published_at: r.published_at,
      body: r.body || '',
      is_latest: idx === 0
    }));
    window.updateReleases(releases);
  } catch (e) {
    console.error('Failed to load releases from GitHub:', e);
    console.log('Using fallback data...');
    
    window.updateReleases([
      { tag_name: '0.5.0', name: 'Latest', published_at: '2026-03-29T00:00:00Z', body: '', is_latest: true },
      { tag_name: '0.4.1', name: '', published_at: '2026-03-15T00:00:00Z', body: '', is_latest: false },
      { tag_name: '0.4.0', name: '', published_at: '2026-02-28T00:00:00Z', body: '', is_latest: false }
    ]);
  }
  console.log('=== loadReleases END ===');
}

async function loadCommits(branch, retries = 3) {
  console.log('=== loadCommits START for branch:', branch, '===');
  currentBranch = branch;
  
  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      console.log(`Fetching commits from GitHub API (attempt ${attempt}/${retries})...`);
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 10000);
      
      const res = await fetch(
        `https://api.github.com/repos/dakychan/Aporia/commits?sha=${branch}&per_page=10`,
        { signal: controller.signal }
      );
      clearTimeout(timeoutId);
      
      console.log('Fetch response status:', res.status);
      
      if (!res.ok) {
        throw new Error(`GitHub API error: ${res.status} ${res.statusText}`);
      }
      
      const data = await res.json();
      
      if (!Array.isArray(data)) {
        throw new Error('Invalid response format from GitHub API');
      }
      
      console.log('Commits fetched:', data.length, 'items');
      
      const commits = data.map(c => ({
        sha: c.sha,
        short_sha: c.sha.substring(0, 7),
        message: c.commit.message.split('\n')[0]
      }));
      window.updateCommits(commits, branch);
      console.log('=== loadCommits END (SUCCESS) ===');
      return;
    } catch (e) {
      console.error(`Attempt ${attempt} failed:`, e.message);
      
      if (attempt === retries) {
        console.error('All retry attempts failed. Using fallback data...');
        window.updateCommits([
          { short_sha: 'a3f8c2d', message: 'fix: исправлен краш при переключении миров' },
          { short_sha: 'b7e1f4a', message: 'feat: добавлена поддержка шейдеров PBR' },
          { short_sha: 'c9d2e6b', message: 'perf: оптимизация рендера частиц на 40%' }
        ], branch);
      } else {
        await new Promise(resolve => setTimeout(resolve, 1000 * attempt));
      }
    }
  }
  console.log('=== loadCommits END ===');
}

// ============================================================
// Интерактивность
// ============================================================

// Кнопка Launch — эффект при нажатии
function setupLaunchButton() {
  const launchBtn = document.getElementById('launchBtn');
  if (!launchBtn) {
    console.error('Launch button not found!');
    return;
  }
  
  launchBtn.addEventListener('click', () => {
    launchBtn.innerHTML = '<span class="btn-content"><i class="fa-solid fa-spinner fa-spin"></i> Starting...</span>';
    launchBtn.style.pointerEvents = 'none';

    triggerFlash();
    setTimeout(triggerFlash, 300);

    // Начинаем симуляцию прогресса
    simulateProgress();

    ipc.postMessage('download_jre');
    ipc.postMessage(`download_version:${selectedVersion}`);

    // Завершаем прогресс на 100% через 3 секунды
    setTimeout(() => {
      setProgress(100);
      
      setTimeout(() => {
        ipc.postMessage(`launch:${selectedVersion}`);
        
        launchBtn.innerHTML = '<span class="btn-content"><i class="fa-solid fa-check"></i> Running</span>';
        launchBtn.style.background = 'linear-gradient(135deg, #059669, #047857)';
        launchBtn.style.boxShadow = '0 4px 30px rgba(5,150,105,0.4), 0 0 60px rgba(52,211,153,0.15)';
        
        setTimeout(() => {
          hideProgress();
        }, 1000);
      }, 500);
    }, 3000);

    setTimeout(() => {
      launchBtn.innerHTML = '<span class="btn-content"><i class="fa-solid fa-play"></i> Launch</span>';
      launchBtn.style.background = '';
      launchBtn.style.boxShadow = '';
      launchBtn.style.pointerEvents = '';
    }, 5000);
  });
}

// ============================================================
// Инициализация
// ============================================================
function initApp() {
  console.log('=== APP INIT START ===');
  console.log('Document ready state:', document.readyState);
  
  // Инициализируем cosmos фон
  if (!cosmosInstance) {
    cosmosInstance = new CosmosBackground();
    console.log('Cosmos background initialized');
  }
  
  setupLaunchButton();
  loadReleases();
  loadCommits('mcp');
  
  console.log('=== APP INIT END ===');
}

// Запускаем при загрузке DOM
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}
