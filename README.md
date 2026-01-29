# c2rust-config

一个用于 c2rust 翻译工作的 Rust 配置管理工具。该工具管理存储在 `.c2rust/config.toml` 文件中的配置数据。

## 安装

```bash
cargo build --release
# 二进制文件将位于 target/release/c2rust-config
```

## 使用方法

该工具提供一个 `config` 子命令, 包含三种配置模式：
- `--global`：全局配置（例如编译器设置）
- `--model`：模型相关配置（例如 AI 模型 API 密钥）
- `--make`：构建/清理/测试相关配置

### 命令结构

```bash
c2rust-config config [模式] [操作] [键] [值...]
```

**模式**（必须指定其中一个）：
- `--global`：全局配置
- `--model`：模型配置
- `--make`：构建/测试配置

**操作**（必须指定其中一个）：
- `--set 键 值...`：设置键的值
- `--unset 键`：删除一个键
- `--add 键 值...`：向数组键添加值
- `--del 键 值...`：从数组键中删除值
- `--list [键]`：列出配置节中的所有值，或列出指定键的值

### 基本示例

#### 完整配置示例

以下是包含所有推荐默认值的完整配置设置：

```bash
# 设置全局配置
c2rust-config config --global --set compiler "gcc"

# 设置完整的构建配置（默认特性）
c2rust-config config --make --set build.dir "build"
c2rust-config config --make --set build.cmd "make"
c2rust-config config --make --set clean.dir "build"
c2rust-config config --make --set clean.cmd "make clean"
c2rust-config config --make --set test.dir "build"
c2rust-config config --make --set test.cmd "make test"
```

#### 单独操作

```bash
# 全局配置
c2rust-config config --global --set compiler "gcc"
c2rust-config config --global --add compiler "clang"
c2rust-config config --global --list

# 模型配置
c2rust-config config --model --set api_key "your-api-key"
c2rust-config config --model --set model_name "gpt-4"
c2rust-config config --model --list

# 构建配置操作
c2rust-config config --make --set build.dir "build"
c2rust-config config --make --set build.cmd "make"
c2rust-config config --make --list

# 删除一个键
c2rust-config config --make --unset build.dir

# 数组操作
c2rust-config config --make --add build.flags "-O2" "-Wall"
c2rust-config config --make --del build.flags "-Wall"

# 数组操作 - 自动类型转换和去重
c2rust-config config --global --set compiler "gcc"       # 设置为字符串
c2rust-config config --global --add compiler "clang"     # 自动转换为数组并添加
c2rust-config config --global --add compiler "gcc"       # 去重：gcc已存在，不会重复添加
c2rust-config config --global --list compiler            # 显示: gcc, clang

# --add 操作的智能行为：
# 1. 如果键是字符串，自动转换为数组
# 2. 添加前自动检查重复，避免相同值多次出现
```

### 特性（Features）

特性允许您为不同的构建场景（例如 debug、release）维护多个配置。默认特性名为 `default`。

**注意**：`--feature` 选项只能与 `--make` 一起使用。

```bash
# 使用特定特性
c2rust-config config --make --feature debug --set build.dir "debug_build"
c2rust-config config --make --feature debug --set build.cmd "make DEBUG=1"

c2rust-config config --make --feature release --set build.dir "release_build"
c2rust-config config --make --feature release --set build.cmd "make RELEASE=1"

# 列出特定特性的配置
c2rust-config config --make --feature debug --list
```

特性名称不区分大小写，会自动转换为小写。

## 配置文件格式

配置存储在 `.c2rust/config.toml` 文件中：

```toml
[global]
# 全局设置，一般无需配置
compiler = ["gcc"]

[model]
# AI 模型相关配置
api_key = "your-api-key"
model_name = "gpt-4"

[feature.default]
# 路径相对于项目根目录（包含 .c2rust 的目录）
"clean.dir" = "build"
"clean.cmd" = "make clean"
"test.dir" = "build"
"test.cmd" = "make test"
"build.dir" = "build"
"build.cmd" = "make"

[feature.debug]
"build.dir" = "debug_build"
"build.cmd" = "make DEBUG=1"
"clean.dir" = "debug_build"
"clean.cmd" = "make clean"
"test.dir" = "debug_build"
"test.cmd" = "make test"
```

**注意**：
- 配置键（如 `build.dir`）在 TOML 文件中可以使用引号（`"build.dir"`）或不使用引号（`build.dir`），两种格式表示同一个键
- 工具会自动处理键的格式，确保键的唯一性，无论文件中使用哪种格式
- 保存配置时，工具会将带点的键自动加上引号以符合 TOML 规范

## 环境变量

### C2RUST_PROJECT_ROOT

该工具支持通过 `C2RUST_PROJECT_ROOT` 环境变量指定项目根目录（包含 `.c2rust` 目录的目录）。

**用途**：
- 允许从任何目录运行工具，而无需先切换到项目目录
- 便于在脚本和自动化工作流中使用
- 支持在多个项目之间快速切换

**使用方法**：

```bash
# 设置环境变量指向项目根目录
export C2RUST_PROJECT_ROOT=/path/to/your/project

# 现在可以从任何目录运行工具
cd /some/other/directory
c2rust-config config --make --list

# 或者在单个命令中设置
C2RUST_PROJECT_ROOT=/path/to/your/project c2rust-config config --global --set compiler "gcc"
```

**默认行为**：
- 如果未设置 `C2RUST_PROJECT_ROOT` 环境变量，工具将使用当前工作目录作为项目根目录
- 这保持了向后兼容性，现有的工作流程无需修改

**示例**：

```bash
# 情况 1：使用环境变量（从任何目录访问配置）
export C2RUST_PROJECT_ROOT=/home/user/my-c2rust-project
cd /tmp
c2rust-config config --make --set build.dir "build"
# 配置将保存到 /home/user/my-c2rust-project/.c2rust/config.toml

# 情况 2：不设置环境变量（使用当前目录）
cd /home/user/my-c2rust-project
c2rust-config config --make --set build.dir "build"
# 配置将保存到当前目录下的 .c2rust/config.toml
```

## 使用要求

- 项目根目录（由 `C2RUST_PROJECT_ROOT` 指定或当前工作目录）中必须存在 `.c2rust` 目录
- 工具不会遍历父目录查找 `.c2rust` 目录
- 如果 `.c2rust` 目录不存在，请手动创建：

```bash
# 在项目根目录中创建
mkdir .c2rust

# 或者使用环境变量指定的路径
# 确保已设置 C2RUST_PROJECT_ROOT 环境变量
: "${C2RUST_PROJECT_ROOT:?请先设置 C2RUST_PROJECT_ROOT 为项目根目录}"
mkdir -p "${C2RUST_PROJECT_ROOT}/.c2rust"
```

- `config.toml` 文件会在首次运行工具时自动创建，包含默认的配置结构

## 错误处理

该工具提供清晰的分层错误消息：

1. **缺少 `.c2rust` 目录**：如果在项目根目录（由 `C2RUST_PROJECT_ROOT` 指定或当前目录）中找不到 `.c2rust` 目录，则显示错误（中文提示）
2. **特性未找到**：尝试访问不存在的特性时
3. **键未找到**：尝试删除或访问不存在的键时
4. **无效操作**：命令语法不正确时（例如缺少必需参数）

**注意**：`config.toml` 文件不存在时会自动创建，包含以下默认结构：
```toml
[global]

[model]

[feature.default]
```

## 验证和警告

该工具会验证特性配置并对不完整的配置发出警告：

**完整的特性配置**：使用 `--make` 配置特性时，应该一起设置以下所有键以形成完整配置：
- `clean.dir` - 要清理的目录
- `clean.cmd` - 清理命令
- `test.dir` - 测试目录
- `test.cmd` - 测试命令
- `build.dir` - 构建输出目录
- `build.cmd` - 构建命令

如果存在这些键中的一部分但不是全部，将显示警告，列出缺少的键。

示例：
```bash
# 不完整的配置 - 将显示警告
c2rust-config config --make --set build.dir "build"
# Warning: Feature 'feature.default' is missing required keys: clean.dir, clean.cmd, test.dir, test.cmd, build.cmd. All of [clean.dir, clean.cmd, test.dir, test.cmd, build.dir, build.cmd] should be configured together.

# 完整的配置 - 无警告
c2rust-config config --make --set build.dir "build"
c2rust-config config --make --set build.cmd "make"
c2rust-config config --make --set clean.dir "build"
c2rust-config config --make --set clean.cmd "make clean"
c2rust-config config --make --set test.dir "build"
c2rust-config config --make --set test.cmd "make test"
```

## 开发

### 运行测试

```bash
cargo test
```

所有测试都是集成测试，位于 `tests/integration_test.rs` 中。

### 项目结构

```
c2rust-config/
├── src/
│   ├── main.rs         # CLI 界面和命令解析
│   ├── config.rs       # 配置文件操作
│   ├── operations.rs   # 核心操作（set、unset、add、del、list）
│   └── error.rs        # 错误处理
├── tests/
│   └── integration_test.rs  # 集成测试
├── Cargo.toml
└── README.md
```

## 许可证

此项目是 c2rust 翻译工具包的一部分。
