# delbin 软件需求规格说明书

**Descriptive Language for Binary Object**

| 项目     | 内容       |
| -------- | ---------- |
| 文档版本 | 1.0.1      |
| 日期     | 2026-01-18 |
| 状态     | 草案       |

---

## 目录

1. [引言](#1-引言)
2. [总体描述](#2-总体描述)
3. [功能需求](#3-功能需求)
4. [外部接口需求](#4-外部接口需求)
5. [非功能需求](#5-非功能需求)
6. [错误处理](#6-错误处理)
7. [附录](#7-附录)

---

## 1. 引言

### 1.1 目的

本文档定义了 Delbin（Descriptive Language for Binary Object）库的软件需求规格。Delbin 是一个用于描述和生成二进制数据结构的领域特定语言（DSL）及其配套工具库，主要用于嵌入式固件打包场景中的头部（Header）信息生成。

本文档的目标读者包括：

- 软件开发人员：实现 Delbin 库
- 固件工程师：使用 Delbin 定义固件头部格式
- 测试人员：验证库功能的正确性
- 项目管理人员：评估项目范围和进度

### 1.2 范围

Delbin 库提供以下核心功能：

- DSL 解析：解析 Delbin 描述语言文本
- 二进制生成：根据 DSL 定义生成二进制数据
- 文件操作：将生成的数据合并到目标文件
- 数据解析：根据 DSL 定义解析现有二进制数据
- 数据验证：验证二进制数据的完整性和正确性

Delbin 库不包含以下功能：

- 固件加密/解密
- 数字签名生成/验证（仅预留接口）
- 固件传输协议
- TLV（Type-Length-Value）动态结构解析

### 1.3 定义、缩略语和缩写

| 术语       | 定义                                               |
| ---------- | -------------------------------------------------- |
| Delbin     | Descriptive Language for Binary Object，本项目名称 |
| DSL        | Domain-Specific Language，领域特定语言             |
| Header     | 固件文件头部，包含元数据信息                       |
| Section    | 数据区段，如 header、image、trailer                |
| CRC        | Cyclic Redundancy Check，循环冗余校验              |
| AST        | Abstract Syntax Tree，抽象语法树                   |
| TOML       | Tom's Obvious Minimal Language，配置文件格式       |
| IEEE 29148 | 软件需求工程标准                                   |

### 1.4 参考文献

| 编号 | 文献                                                         |
| ---- | ------------------------------------------------------------ |
| [1]  | IEEE Std 29148-2018, Systems and software engineering — Life cycle processes — Requirements engineering |
| [2]  | MCUboot Documentation, https://docs.mcuboot.com              |
| [3]  | TOML Specification v1.0.0, https://toml.io                   |
| [4]  | Pest Parser Documentation, https://pest.rs                   |
| [5]  | CRC RevEng Catalogue, https://reveng.sourceforge.io/crc-catalogue |

---

## 2. 总体描述

### 2.1 产品愿景

Delbin 旨在提供一种声明式的方式来描述二进制数据结构，使嵌入式固件工程师能够：

- 使用人类可读的语法定义固件头部格式
- 自动计算大小、偏移、CRC 等字段
- 在不同项目间复用头部定义
- 验证现有固件文件的头部信息

### 2.2 产品功能概述

### 2.3 用户特征

| 用户类型 | 特征描述 |
|----------|----------|
| 固件工程师 | 熟悉嵌入式开发，了解二进制数据结构，需要定义固件头部格式 |
| 工具开发者 | 熟悉 Rust 编程，需要将 Delbin 集成到打包工具中 |
| CI/CD 工程师 | 需要在自动化流程中使用 Delbin 生成固件包 |

### 2.4 约束条件

| 约束 | 描述 |
|------|------|
| 实现语言 | Rust |
| 解析器 | 使用 pest 库实现 |
| 配置格式 | DSL 文本嵌入 TOML 配置文件 |
| 目标平台 | 跨平台（Linux、Windows、macOS） |

### 2.5 假设与依赖

| 假设/依赖 | 描述 |
|-----------|------|
| 环境变量 | 调用方负责在调用前解析并提供所有环境变量 |
| 文件访问 | 调用方确保输入文件可读、输出路径可写 |
| 字节序 | 目标平台支持指定的字节序 |
| 外部数据 | image 等外部 section 数据由调用方提供 |

---

## 3. 功能需求

### 3.1 DSL 语法规范

#### 3.1.1 全局指令

**FR-DSL-001: 字节序指令**

DSL 应支持全局字节序指令，语法如下：

```text
@endian = little; // 小端序 @endian = big; // 大端序
```
| 属性 | 说明 |
|------|------|
| 默认值 | little |
| 作用范围 | 整个 DSL 文件 |
| 出现次数 | 最多一次，必须在 struct 定义之前 |

#### 3.1.2 结构体定义

**FR-DSL-002: 结构体声明**

DSL 应支持结构体定义，语法如下：
```text
struct [attributes] { <field_definitions> }
```

**FR-DSL-003: 结构体属性**

| 属性 | 语法 | 说明 |
|------|------|------|
| packed | `@packed` | 紧凑布局，无填充 |
| align | `@align(n)` | 按 n 字节对齐 |

示例：
```text
struct header @packed { ... } struct header @align(4) { ... }
```

#### 3.1.3 字段定义

**FR-DSL-004: 字段声明语法**

字段定义语法如下：

```
<name>: <type> [= <expression>];
```
其中：
- `name`：字段名称，符合标识符规则
- `type`：字段类型
- `expression`：可选的初始化表达式

#### 3.1.4 类型系统

**FR-DSL-005: 标量类型**

| 类型 | 大小 | 说明 |
|------|------|------|
| u8 | 1 字节 | 无符号 8 位整数 |
| u16 | 2 字节 | 无符号 16 位整数 |
| u32 | 4 字节 | 无符号 32 位整数 |
| u64 | 8 字节 | 无符号 64 位整数 |
| i8 | 1 字节 | 有符号 8 位整数 |
| i16 | 2 字节 | 有符号 16 位整数 |
| i32 | 4 字节 | 有符号 32 位整数 |
| i64 | 8 字节 | 有符号 64 位整数 |

**FR-DSL-006: 数组类型**

数组类型使用 Rust 风格语法：
```
[<scalar_type>; <len>]
```
示例：
```text
magic: [u8; 4] // 4 字节数组 
padding: [u8; 32] // 32 字节数组 
data: [u32; 8] // 8 个 u32 元素的数组
```
数组初始化语法

| 语法                      | 说明                                 |
| :------------------------ | :----------------------------------- |
| `[u8; N]`                 | 默认填充 0x00                        |
| `[u8; N] = [val; N]`      | 填充指定值（完整形式）               |
| `[u8; N] = [val; _]`      | 填充指定值（简化形式，自动推断大小） |
| `[u8; N] = [a, b, c]`     | 逐元素指定，不足部分补 0x00          |
| `[u8; N] = @bytes("str")` | 函数返回值                           |

#### 3.1.5 表达式

**FR-DSL-007: 字面量**

| 类型 | 语法 | 示例 |
|------|------|------|
| 十进制 | `[0-9]+` | `12345` |
| 十六进制 | `0x[0-9a-fA-F]+` | `0xDEADBEEF` |
| 二进制 | `0b[01]+` | `0b10101010` |
| 字符串 | `"..."` | `"hello"` |

**FR-DSL-008: 环境变量引用**

语法：`${VARIABLE_NAME}`

环境变量由调用方在执行前注入，DSL 中引用未定义的环境变量将产生错误。

示例：
```
version: u32 = ${VERSION_MAJOR}; 
timestamp: u32 = ${UNIX_STAMP};
```

**FR-DSL-009: 标志位常量**

预定义标志位常量采用全大写命名：

| 常量 | 值 | 说明 |
|------|-----|------|
| FLAG_SIGNED | 0x01 | 已签名 |
| FLAG_ENCRYPTED | 0x02 | 已加密 |
| FLAG_COMPRESSED | 0x04 | 已压缩 |
| FLAG_RAM_LOAD | 0x08 | RAM 加载 |

用户可通过环境变量定义自定义标志位。

**FR-DSL-010: 运算符**

| 运算符 | 说明 | 优先级 |
|--------|------|--------|
| `()` | 括号 | 最高 |
| `~` | 位取反 | 高 |
| `<<` | 左移 | 中 |
| `>>` | 右移 | 中 |
| `&` | 位与 | 低 |
| `\|` | 位或 | 最低 |

示例：
```
flags: u32 = FLAG_SIGNED | FLAG_ENCRYPTED; 
version: u32 = (${MAJOR} << 24) ∣ (${MINOR} << 16) | ${PATCH}; 
mask: u32 = ~0x0F;
```

### 3.2 内置函数

#### 3.2.1 字节转换函数

**FR-FUNC-001: @bytes()**

将字符串转换为字节数组。

| 属性 | 说明 |
|------|------|
| 语法 | `@bytes(<string>)` |
| 参数 | string: 字符串字面量或环境变量 |
| 返回 | 字节数组 |
| 行为 | 字符串不足时用 0x00 填充，超长时截断并产生警告 |

示例：
```
magic: [u8; 4] = @bytes("FPK"); // [0x46, 0x50, 0x4B, 0x00] 
partition: [u8; 16] = @bytes("app"); // "app" + 13 个 0x00 
name: [u8; 8] = @bytes(${NAME}); // 从环境变量获取
```

#### 3.2.2 大小计算函数

**FR-FUNC-002: @sizeof()**

计算 section 或结构体的大小。

| 属性 | 说明 |
|------|------|
| 语法 | `@sizeof(<section>)` |
| 参数 | section: section 名称或 `@self` |
| 返回 | u32，字节大小 |

示例：
```
img_size: u32 = @sizeof(image); // image section 的大小 
header_size: u32 = @sizeof(@self); // 当前结构体的大小 
total_size: u32 = @sizeof(header) + @sizeof(image);
```

**FR-FUNC-003: @offsetof()**

计算字段在结构体中的偏移量。

| 属性 | 说明 |
|------|------|
| 语法 | `@offsetof(<field_name>)` |
| 参数 | field_name: 当前结构体中的字段名 |
| 返回 | u32，字节偏移量 |

示例：
```text
crc_offset: u32 = @offsetof(header_crc); // header_crc 字段的偏移量
_pad: [u8; 128 - @offsetof(_pad)];  // 自引用，返回当前偏移
```

#### 3.2.3 版本打包函数

**FR-FUNC-004: @version_pack()**

（该需求已被移除）

#### 3.2.4 校验函数

**FR-FUNC-005: @crc32()**

计算 CRC32 校验值（ISO-HDLC 算法）。

| 属性 | 说明 |
|------|------|
| 语法 | `@crc32(<range>)` |
| 参数 | range: section 引用或范围表达式 |
| 返回 | u32 |
| 算法 | CRC32-ISO-HDLC (poly=0x04C11DB7, init=0xFFFFFFFF, xorout=0xFFFFFFFF, refin=true, refout=true) |

**FR-FUNC-006: @crc16()**

计算 CRC16 校验值（CCITT 算法）。

| 属性 | 说明 |
|------|------|
| 语法 | `@crc16(<range>)` |
| 参数 | range: section 引用或范围表达式 |
| 返回 | u16 |
| 算法 | CRC16-CCITT (poly=0x1021, init=0xFFFF, xorout=0x0000, refin=false, refout=false) |

**FR-FUNC-007: @crc() 通用函数**

使用指定算法计算 CRC。

| 属性 | 说明 |
|------|------|
| 语法 | `@crc(<algorithm>, <range>)` |
| 参数 | algorithm: 算法名称字符串; range: 数据范围 |
| 返回 | 根据算法返回 u16 或 u32 |

支持的算法：

| 算法名 | 输出类型 | 说明 |
|--------|----------|------|
| "crc32" | u32 | CRC32-ISO-HDLC |
| "crc32-mpeg2" | u32 | CRC32-MPEG2 |
| "crc16" | u16 | CRC16-CCITT |
| "crc16-modbus" | u16 | CRC16-MODBUS |

示例：
```
crc1: u32 = @crc32(image); // 专用函数 crc2: u16 = @crc("crc16-modbus", image); // 通用函数
```


**FR-FUNC-008: @sha256()**

计算 SHA256 哈希值。

| 属性 | 说明 |
|------|------|
| 语法 | `@sha256(<range>)` |
| 参数 | range: section 引用或范围表达式 |
| 返回 | [u8; 32] |

**FR-FUNC-009: @hash() 通用函数**

使用指定算法计算哈希。

| 属性 | 说明 |
|------|------|
| 语法 | `@hash(<algorithm>, <range>)` |
| 参数 | algorithm: 算法名称; range: 数据范围 |
| 返回 | 字节数组 |

支持的算法：

| 算法名 | 输出大小 |
|--------|----------|
| "sha256" | 32 字节 |
| "sha1" | 20 字节 |
| "md5" | 16 字节 |

#### 3.2.5 范围表达式

**FR-FUNC-010: 范围语法**

用于指定计算校验值的数据范围。

| 语法 | 说明 |
|------|------|
| `<section>` | 整个 section |
| `@self` | 当前结构体 |
| `@self[<start>..<end>]` | 当前结构体的字节切片 |
| `@self[..<field>]` | 从开头到指定字段之前 |
| `@self[<field>..]` | 从指定字段开始到结尾 |

示例：
```
// 计算 image section 的 CRC img_crc: 
u32 = @crc32(image);
// 计算从 header 开头到 header_crc 字段之前的 CRC header_crc: 
u32 = @crc32(@self[..header_crc]);
// 计算 header 和 image 组合的哈希 
combined_hash: [u8; 32] = @sha256(header, image);
```
**FR-FUNC-011: 自引用 CRC 计算**

当 CRC 字段需要计算包含自身之前所有字段的校验值时，采用两阶段计算：

第一阶段：
1. 按顺序计算所有非自引用字段
2. 自引用 CRC 字段暂时填充为 0

第二阶段：
1. 计算指定范围的 CRC
2. 将结果回填到 CRC 字段

### 3.3 默认值规则

**FR-DEFAULT-001: 字段默认值**

| 类型 | 未指定值时的默认行为 |
|------|----------------------|
| 标量类型 | 填充 0x00 |
| 数组类型 | 所有元素填充 0x00 |

示例：
```text
reserved1: [u8; 8]; // 8 个 0x00 
reserved2: [u8; 8] = [0xFF; _]; // 8 个 0xFF 
padding: u32; // 0x00000000
```

### 3.4 库 API 接口

#### 3.4.1 生成接口

**FR-API-001: generate()**

根据 DSL 定义生成二进制数据。

输入参数：
- dsl_text: DSL 描述文本
- env: 环境变量映射表
- sections: 外部 section 数据映射表

输出选项：
- 二进制字节流 (Vec<u8>)
- 十六进制字符串 (String)
- 写入文件 (指定路径)

```rust
pub struct GenerateOptions {
    pub output_format: OutputFormat,
    pub output_path: Option<PathBuf>,
}

pub enum OutputFormat {
    Binary,      // Vec<u8>
    HexString,   // 纯十六进制字符串，如 "DEADBEEF"
    File,        // 写入文件
}

pub fn generate(
    dsl_text: &str,
    env: &HashMap<String, Value>,
    sections: &HashMap<String, Vec<u8>>,
    options: &GenerateOptions,
) -> Result<GenerateResult, DelBinError>;
```

#### 3.4.2 合并接口
**FR-API-002: merge()**

将生成的 header 合并到目标文件头部。

输入参数：

- dsl_text: DSL 描述文本
- env: 环境变量映射表
- target_file: 目标文件路径
- output_file: 输出文件路径
- 
行为：

- 解析 DSL 并生成 header 数据
- 读取目标文件作为 image section
- 将 header + image 写入输出文件

```rust
pub fn merge(
    dsl_text: &str,
    env: &HashMap<String, Value>,
    target_file: &Path,
    output_file: &Path,
) -> Result<MergeResult, DelBinError>;

```

#### 3.4.3 解析接口
**FR-API-003: parse()**

根据 DSL 定义解析现有二进制数据。
输入参数：

- dsl_text: DSL 描述文本（作为 schema）
- data: 二进制数据

输出：

- 解析后的字段-值映射表

```rust
pub fn parse(
    dsl_text: &str,
    data: &[u8],
) -> Result<ParseResult, DelBinError>;

pub struct ParseResult {
    pub fields: HashMap<String, FieldValue>,
    pub raw_bytes: Vec<u8>,
}

pub enum FieldValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Bytes(Vec<u8>),
}
```

#### 3.4.4 验证接口
**FR-API-004: validate()**

验证二进制数据是否符合 DSL 定义。

输入参数：

- dsl_text: DSL 描述文本
- data: 待验证的二进制数据
- sections: 用于校验计算的外部 section（如 image）

验证项目：

- Magic 值匹配
- CRC 校验正确
- Hash 校验正确
- 大小字段与实际一致

```rust
pub fn validate(
    dsl_text: &str,
    data: &[u8],
    sections: &HashMap<String, Vec<u8>>,
) -> Result<ValidationResult, DelBinError>;

pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}
```

## 4. 外部接口需求
### 4.1 用户接口
**EIR-UI-001: DSL 文本格式**

DSL 文本采用 UTF-8 编码，嵌入 TOML 配置文件的多行字符串中。

DSL 文本字符串示例：
```text
"""
@endian = little;

struct header @packed {
    magic:        [u8; 4] = @bytes("fpk\0");
    version:      u32 = @version_pack(${VERSION_MAJOR}, ${VERSION_MINOR}, ${VERSION_PATCH});
    img_size:     u32 = @sizeof(image);
    header_crc:   u32 = @crc32(@self[..header_crc]);
}
"""
```
### 4.2 软件接口
**EIR-SI-001: Rust Crate 接口**

Delbin 作为 Rust crate 发布，提供以下模块：

| 模块                | 功能             |
| :------------------ | :--------------- |
| `delbin::parser`    | DSL 解析器       |
| `delbin::generator` | 二进制数据生成器 |
| `delbin::reader`    | 二进制数据解析器 |
| `delbin::validator` | 数据验证器       |
| `delbin::error`     | 错误类型定义     |

### 4.3 数据格式
EIR-DF-001: 输出格式

| 格式      | 说明                                                  |
| :-------- | :---------------------------------------------------- |
| Binary    | 原始二进制字节流                                      |
| HexString | 纯十六进制字符串，大写，无分隔符，如 `"4650 4B00..."` |
| File      | 二进制文件                                            |

## 5. 非功能需求
### 5.1 性能需求
**NFR-PERF-001: 解析性能**

DSL 解析时间应满足：

- 1KB DSL 文本：< 10ms
- 10KB DSL 文本：< 100ms

**NFR-PERF-002: 生成性能**

二进制数据生成时间应满足：

- 1KB header + 1MB image：< 100ms
- CRC32 计算：> 100 MB/s

### 5.2 可靠性需求

**NFR-REL-001: 错误处理**

所有可预见的错误情况必须有明确的错误码和错误信息，不允许 panic。

**NFR-REL-002: 数据完整性**

生成的二进制数据必须与 DSL 定义完全一致，CRC/Hash 计算必须正确。

### 5.3 可维护性需求

**NFR-MAIN-001: 代码文档**

所有公开 API 必须有完整的 rustdoc 文档。

**NFR-MAIN-002: 测试覆盖**

单元测试覆盖率应达到 80% 以上。

### 5.4 可移植性需求

**NFR-PORT-001: 平台支持**

库应支持以下平台：

- Linux (x86_64, aarch64)
- Windows (x86_64)
- macOS (x86_64, aarch64)

## 6. 错误处理
### 6.1 错误码格式
错误码格式：EXXYYY

XX: 类别码 (01-99)
YYY: 具体错误 (001-999)

### 6.2 错误类别

| 类别码 | 类别名称   | 说明             |
| :----- | :--------- | :--------------- |
| 01     | Parse      | DSL 语法解析错误 |
| 02     | Semantic   | 语义分析错误     |
| 03     | Type       | 类型相关错误     |
| 04     | Eval       | 表达式求值错误   |
| 05     | IO         | 文件操作错误     |
| 06     | Validation | 验证错误         |

### 6.3 详细错误码

#### 6.3.1 解析错误 (01)

| 错误码 | 名称            | 说明           |
| :----- | :-------------- | :------------- |
| E01001 | UnexpectedToken | 意外的 token   |
| E01002 | UnexpectedEOF   | 意外的文件结束 |
| E01003 | InvalidSyntax   | 无效语法       |
| E01004 | InvalidNumber   | 无效数字格式   |
| E01005 | InvalidString   | 无效字符串格式 |
| E01006 | UnclosedBracket | 未闭合的括号   |
| E01007 | UnclosedString  | 未闭合的字符串 |

#### 6.3.2 语义错误 (02)

| 错误码 | 名称               | 说明                     |
| :----- | :----------------- | :----------------------- |
| E02001 | UndefinedVariable  | 未定义的环境变量         |
| E02002 | UndefinedField     | 未定义的字段引用         |
| E02003 | UndefinedSection   | 未定义的 section 引用    |
| E02004 | UndefinedFunction  | 未定义的内置函数         |
| E02005 | DuplicateField     | 重复的字段定义           |
| E02006 | DuplicateStruct    | 重复的结构体定义         |
| E02007 | InvalidReference   | 无效的引用（如前向引用） |
| E02008 | CircularDependency | 循环依赖                 |

#### 6.3.3 类型错误 (03)

| 错误码 | 名称              | 说明               |
| :----- | :---------------- | :----------------- |
| E03001 | TypeMismatch      | 类型不匹配         |
| E03002 | ArraySizeMismatch | 数组大小不匹配     |
| E03003 | IntegerOverflow   | 整数溢出           |
| E03004 | InvalidArraySize  | 无效的数组大小     |
| E03005 | StringTooLong     | 字符串超过数组长度 |
| E03006 | InvalidCast       | 无效的类型转换     |

#### 6.3.4 求值错误 (04)

| 错误码 | 名称                  | 说明             |
| :----- | :-------------------- | :--------------- |
| E04001 | DivisionByZero        | 除零错误         |
| E04002 | InvalidRange          | 无效的范围       |
| E04003 | InvalidArgument       | 函数参数无效     |
| E04004 | ArgumentCountMismatch | 函数参数数量错误 |
| E04005 | ComputationFailed     | 计算失败         |
| E04006 | ShiftOverflow         | 移位溢出         |

#### 6.3.5 IO 错误 (05)

| 错误码 | 名称             | 说明           |
| :----- | :--------------- | :------------- |
| E05001 | FileNotFound     | 文件不存在     |
| E05002 | FileReadError    | 文件读取失败   |
| E05003 | FileWriteError   | 文件写入失败   |
| E05004 | InvalidFilePath  | 无效的文件路径 |
| E05005 | PermissionDenied | 权限不足       |

#### 6.3.6 验证错误 (06)

| 错误码 | 名称            | 说明               |
| :----- | :-------------- | :----------------- |
| E06001 | MagicMismatch   | Magic 值不匹配     |
| E06002 | CrcMismatch     | CRC 校验失败       |
| E06003 | HashMismatch    | Hash 校验失败      |
| E06004 | SizeMismatch    | 大小字段与实际不符 |
| E06005 | VersionMismatch | 版本不匹配         |
| E06006 | InvalidHeader   | 无效的 header 结构 |

### 6.4 警告码

| 警告码 | 名称            | 说明               |
| :----- | :-------------- | :----------------- |
| W03001 | StringTruncated | 字符串被截断       |
| W03002 | ValueTruncated  | 数值被截断         |
| W04001 | UnusedVariable  | 定义了未使用的变量 |
| W04002 | UnusedField     | 定义了未使用的字段 |

### 6.5 错误处理行为

| 情况               | 行为                  |
| :----------------- | :-------------------- |
| 字符串超过数组长度 | 截断，产生警告 W03001 |
| 环境变量未定义     | 报错 E02001           |
| CRC 计算范围无效   | 报错 E04002           |
| 数值溢出           | 报错 E03003           |

### 6.6 错误信息格式

```rust
pub struct DelBinError {
    pub code: ErrorCode,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub hint: Option<String>,
}

pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub context: String,  // 出错行的内容
}
```

错误信息示例：
```text
error[E02001]: Undefined variable 'VERSION_MAJOR'
```



## 7. 附录

### 7.1 完整语法示例

```text
// 字节序声明
@endian = little;

// 结构体定义
struct header @packed {
    // ===== 标识区 =====
    // 魔数
    magic:          [u8; 4] = @bytes("fpk\0");
    
    // 头部版本
    header_ver:     u16 = 0x0100;
    
    // 头部大小
    header_size:    u16 = @sizeof(@self);
    
    // ===== 版本区 =====
    
    // 固件版本（自定义位运算）
    fw_version: u32 = (${VERSION_MAJOR} << 24) | (${VERSION_MINOR} << 16) | ${VERSION_PATCH};
    
    // 构建号
    build_number:   u32 = ${BUILD_NUMBER};
    
    // 版本字符串
    version_str:    [u8; 16] = @bytes(${VERSION_STRING});
    
    // ===== 标志区 =====
    // 配置标志
    flags:          u32 = FLAG_SIGNED | FLAG_ENCRYPTED;
    
    // ===== 大小区 =====
    // 镜像大小
    img_size:       u32 = @sizeof(image);
    
    // 打包大小（用于压缩场景）
    packed_size:    u32 = @sizeof(image);
    
    // ===== 时间区 =====
    // 构建时间戳
    timestamp:      u32 = ${UNIX_STAMP};
    
    // ===== 描述区 =====
    // 分区名称
    partition:      [u8; 16] = @bytes("app");
    
    // 水印
    watermark:      [u8; 16] = @bytes("DELBIN_DEMO");
    
    // ===== 保留区 =====
    reserved:       [u8; 32];
    
    // ===== 校验区 =====
    // 镜像 CRC
    img_crc32:      u32 = @crc32(image);
    
    // 镜像 SHA256
    img_sha256:     [u8; 32] = @sha256(image);
    
    // 头部 CRC（自引用）
    header_crc32:   u32 = @crc32(@self[..header_crc32]);

    // 填充到 256 字节总长
    _padding:          [u8; 256 - @offsetof(_padding)];
}
```

### 7.2 EBNF 语法定义
```
(* 顶层结构 *)
file            = { directive } , { struct_def } ;

(* 全局指令 *)
directive       = "@" , directive_name , "=" , directive_value , ";" ;
directive_name  = "endian" ;
directive_value = "little" | "big" ;

(* 结构体定义 *)
struct_def      = "struct" , identifier , { struct_attr } , "{" , { field_def } , "}" ;
struct_attr     = "@packed" | ( "@align" , "(" , number , ")" ) ;

(* 字段定义 *)
field_def       = identifier , ":" , type_spec , [ "=" , expression ] , ";" ;

(* 类型 *)
type_spec       = scalar_type | array_type ;
scalar_type     = ( "u" | "i" ) , ( "8" | "16" | "32" | "64" ) ;
array_type      = "[" , scalar_type , ";" , number , "]" ;

(* 表达式 *)
expression      = or_expr ;
or_expr         = and_expr , { "|" , and_expr } ;
and_expr        = shift_expr , { "&" , shift_expr } ;
shift_expr      = unary_expr , { ( "<<" | ">>" ) , unary_expr } ;
unary_expr      = [ "~" ] , primary_expr ;
primary_expr    = number | string | env_var | flag | builtin_call | "(" , expression , ")" ;

(* 字面量 *)
number          = decimal | hexadecimal | binary ;
decimal         = digit , { digit } ;
hexadecimal     = "0x" , hex_digit , { hex_digit } ;
binary          = "0b" , ( "0" | "1" ) , { "0" | "1" } ;
string          = '"' , { string_char } , '"' ;

(* 环境变量 *)
env_var         = "${" , identifier , "}" ;

(* 标志常量 *)
flag            = upper_letter , { upper_letter | "_" | digit } ;

(* 内置函数调用 *)
builtin_call    = "@" , builtin_name , "(" , [ arg_list ] , ")" ;
builtin_name    = "bytes" | "sizeof" | "offsetof" 
                | "crc32" | "crc16" | "crc" | "sha256" | "hash" ;
arg_list        = argument , { "," , argument } ;
argument        = expression | range_expr | section_ref ;

(* 范围表达式 *)
range_expr      = "@self" , [ "[" , range_spec , "]" ] ;
range_spec      = [ range_start ] , ".." , [ range_end ] ;
range_start     = number | identifier ;
range_end       = number | identifier ;

(* Section 引用 *)
section_ref     = identifier ;

(* 标识符 *)
identifier      = ( letter | "_" ) , { letter | digit | "_" } ;

(* 基本字符 *)
letter          = "a" | ... | "z" | "A" | ... | "Z" ;
upper_letter    = "A" | ... | "Z" ;
digit           = "0" | ... | "9" ;
hex_digit       = digit | "a" | ... | "f" | "A" | ... | "F" ;
string_char     = ? any character except '"' and newline ? | escape_seq ;
escape_seq      = "\\" , ( "n" | "r" | "t" | "\\" | '"' | "0" | ( "x" , hex_digit , hex_digit ) ) ;

```

### 7.3 内置函数速查表

| 函数      | 语法                 | 返回类型 | 说明                                 |
| :-------- | :------------------- | :------- | :----------------------------------- |
| @bytes    | `@bytes(string)`     | [u8; N]  | 字符串转字节数组                     |
| @sizeof   | `@sizeof(section)`   | u32      | 计算大小                             |
| @offsetof | `@offsetof(field)`   | u32      | 计算偏移量，支持自引用，返回当前偏移 |
| @crc32    | `@crc32(range)`      | u32      | CRC32-ISO-HDLC                       |
| @crc16    | `@crc16(range)`      | u16      | CRC16-CCITT                          |
| @crc      | `@crc(algo, range)`  | u16/u32  | 通用 CRC                             |
| @sha256   | `@sha256(range)`     | [u8; 32] | SHA256 哈希                          |
| @hash     | `@hash(algo, range)` | [u8; N]  | 通用哈希                             |

### 7.4 预定义标志位
暂不实现

### 7.5 CRC 算法参数

| 算法             | 多项式     | 初始值     | 输出异或   | RefIn | RefOut |
| :--------------- | :--------- | :--------- | :--------- | :---- | :----- |
| crc32 (ISO-HDLC) | 0x04C11DB7 | 0xFFFFFFFF | 0xFFFFFFFF | true  | true   |
| crc32-mpeg2      | 0x04C11DB7 | 0xFFFFFFFF | 0x00000000 | false | false  |
| crc16 (CCITT)    | 0x1021     | 0xFFFF     | 0x0000     | false | false  |
| crc16-modbus     | 0x8005     | 0xFFFF     | 0x0000     | true  | true   |

## 文档历史

| 版本  | 日期       | 作者   | 变更说明 |
| :---- | :--------- | :----- | :------- |
| 1.0.0 | 2026-01-18 | nasihs | 初始版本 |
| 1.1.0 | 2026-01-18 | nasihs |          |

---

*本文档依据 IEEE Std 29148-2018 标准编写*



