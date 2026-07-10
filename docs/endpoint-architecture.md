# JM API 多端点架构设计

## 现状分析

JM 有多个可用的 API 端点，现有 Tauri 版本实现了：

1. **备用端点列表**
   - `https://www.cdnhjk.net`（默认）
   - `https://www.cdnhth.club`

2. **动态端点发现**
   - 从远程配置文件获取最新端点列表
   - 配置文件地址（加密存储）：
     - `https://rup4a04-c02.tos-cn-hongkong.bytepluses.com/newsvr-2025.txt`
     - `https://rup4a04-c01.tos-ap-southeast-1.bytepluses.com/newsvr-2025.txt`
   - AES 解密密钥种子：`diosfjckwpqpdfjkvnqQjsik`

3. **端点探测 (Endpoint Probing)**
   - 并发测试所有候选端点
   - 记录延迟和可用性
   - 按延迟排序，可用的排在前面
   - 缓存探测结果 30 分钟

## 推荐架构

### 方案 A：服务端统一管理（推荐）⭐

**优点：**
- 客户端无感知，自动切换最优端点
- 集中管理端点状态，减少重复探测
- 支持健康检查和自动故障转移
- 可以实现负载均衡

**缺点：**
- 服务端复杂度稍高
- 需要后台任务定期探测

```
┌─────────────┐
│  Web 前端   │
└──────┬──────┘
       │ HTTP API (透明)
┌──────▼──────────────────┐
│   Rust 后端             │
│  ┌─────────────────┐    │
│  │ 端点管理器      │    │
│  │ - 动态发现      │    │
│  │ - 健康检查      │    │
│  │ - 自动切换      │    │
│  └─────────────────┘    │
│         │                │
│    ┌────┴────┐          │
│    ▼         ▼          │
│  JM-A     JM-B  ...     │
└─────────────────────────┘
```

**实现要点：**
```rust
// server/src/jm/endpoint_manager.rs
pub struct EndpointManager {
    endpoints: Arc<RwLock<Vec<EndpointInfo>>>,
    current: Arc<RwLock<String>>,
}

impl EndpointManager {
    // 启动时发现端点
    pub async fn discover() -> Result<Self>;
    
    // 后台任务：每 30 分钟探测一次
    pub async fn start_health_check_loop(&self);
    
    // 获取当前最优端点（对外透明）
    pub async fn get_best_endpoint(&self) -> String;
    
    // 端点请求失败时自动切换
    pub async fn mark_failed(&self, endpoint: &str);
}
```

### 方案 B：客户端选择

**优点：**
- 用户可手动选择端点
- 后端实现简单

**缺点：**
- 前端需要处理端点选择逻辑
- 每个客户端都要探测，浪费资源

```
┌─────────────────────────┐
│  Web 前端               │
│  ┌──────────────────┐   │
│  │ 端点选择器       │   │
│  │ - 手动/自动      │   │
│  └──────────────────┘   │
└───┬──────────┬──────────┘
    │          │
┌───▼──┐   ┌──▼───┐
│ JM-A │   │ JM-B │ ...
└──────┘   └──────┘
```

### 方案 C：混合模式（最佳 🎯）

**结合两者优点：**
- 服务端默认自动管理（方案 A）
- 前端提供手动切换选项（用户偏好）
- 支持测速 API，供前端展示

```
┌─────────────────────────────┐
│  Web 前端                   │
│  ┌──────────────────────┐   │
│  │ 设置页面             │   │
│  │ ○ 自动选择（默认）   │   │
│  │ ○ 手动：JM-A (120ms)│   │
│  │          JM-B (350ms)│   │
│  └──────────────────────┘   │
└────────┬────────────────────┘
         │ API (带端点偏好)
┌────────▼─────────────────────┐
│  Rust 后端                   │
│  ┌──────────────────────┐    │
│  │ 端点管理器           │    │
│  │ - 自动发现 + 探测    │    │
│  │ - 尊重用户偏好       │    │
│  │ - 提供测速 API       │    │
│  └──────────────────────┘    │
└──────────────────────────────┘
```

## 推荐实现（方案 C）

### 后端 API 设计

```rust
// GET /api/endpoints - 获取所有端点及状态
{
  "endpoints": [
    {
      "url": "https://www.cdnhjk.net",
      "available": true,
      "latency_ms": 120,
      "last_check": "2026-07-10T10:30:00Z"
    },
    {
      "url": "https://www.cdnhth.club",
      "available": true,
      "latency_ms": 350,
      "last_check": "2026-07-10T10:30:00Z"
    }
  ],
  "recommended": "https://www.cdnhjk.net"
}

// POST /api/endpoints/probe - 主动触发探测
// GET /api/comics?endpoint=custom - 可选指定端点
```

### 文件结构

```
server/src/
├── jm/                      # JM API 相关
│   ├── mod.rs
│   ├── client.rs           # HTTP 客户端
│   ├── endpoint_manager.rs # 端点管理器（核心）
│   ├── discovery.rs        # 端点发现（解密配置）
│   ├── models.rs           # API 数据模型
│   └── auth.rs             # 认证
├── api/
│   ├── endpoints.rs        # 端点管理 API
│   ├── comics.rs           # 透明调用 JM
│   └── ...
└── main.rs
```

### 配置优先级

```
用户手动指定 > 配置文件 > 自动探测结果 > 默认端点
```

## 迁移计划

### Phase 1: 基础端点管理
1. 迁移常量和配置
2. 实现端点发现逻辑（含解密）
3. 实现端点探测

### Phase 2: 后端集成
1. 创建 `EndpointManager`
2. 启动时自动发现
3. 后台健康检查任务

### Phase 3: API 暴露
1. `GET /api/endpoints` - 端点列表
2. `POST /api/endpoints/probe` - 主动探测
3. 所有 JM API 使用最优端点

### Phase 4: 前端对接
1. 设置页面显示端点列表
2. 支持手动选择
3. 显示延迟状态

## 关键代码框架

```rust
// server/src/jm/endpoint_manager.rs
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EndpointManager {
    endpoints: Arc<RwLock<Vec<EndpointInfo>>>,
    current: Arc<RwLock<String>>,
    client: reqwest::Client,
}

#[derive(Clone, Debug)]
pub struct EndpointInfo {
    pub url: String,
    pub available: bool,
    pub latency_ms: Option<u64>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

impl EndpointManager {
    pub async fn new() -> Result<Self> {
        let manager = Self {
            endpoints: Arc::new(RwLock::new(vec![])),
            current: Arc::new(RwLock::new(DEFAULT_ENDPOINT.to_string())),
            client: reqwest::Client::new(),
        };
        
        // 初始发现
        manager.discover().await?;
        
        Ok(manager)
    }
    
    // 发现所有可用端点
    async fn discover(&self) -> Result<()> {
        let mut candidates = FALLBACK_ENDPOINTS.to_vec();
        
        // 从远程配置获取
        if let Ok(remote) = fetch_remote_config(&self.client).await {
            candidates.extend(remote);
        }
        
        // 探测所有端点
        let probes = self.probe_all(&candidates).await;
        
        *self.endpoints.write().await = probes;
        self.update_current().await;
        
        Ok(())
    }
    
    // 获取当前最优端点
    pub async fn get_endpoint(&self) -> String {
        self.current.read().await.clone()
    }
    
    // 后台健康检查
    pub fn start_background_check(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(30 * 60)
            );
            
            loop {
                interval.tick().await;
                let _ = self.discover().await;
            }
        });
    }
}
```

## 总结

**推荐：方案 C（混合模式）**

- 后端自动管理，用户无感知
- 提供手动选择，满足特殊需求
- 实现简单，性能最优
- 易于扩展（负载均衡、故障转移）

你觉得这个方案如何？需要我开始实现吗？
