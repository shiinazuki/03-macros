# 第三课 · 宏 学习笔记

## 做出来的东西

- `src/decl.rs` —— 声明宏 `my_vec!`，三条规则，7 个测试
- `crates/macros-derive/` —— 过程宏 crate，一个 `#[derive(TypeName)]`
- `src/enum_from.rs` —— 过程宏的使用现场

---

## 一、声明宏 `macro_rules!`

### 语法速查

```rust
macro_rules! 名字 {
    (匹配模式) => { 展开成的代码 };   // 多条规则自上而下匹配，第一条命中的胜出
}
```

`$( $x:expr ),+` 四个部分：

| 部分 | 说明 |
|---|---|
| `$( ... )` | 重复组。`$` 后面跟括号 = 重复组，跟标识符 = 元变量 |
| `$x:expr` | 元变量。`$` 贴在**自己起的名字**上，冒号后是**片段类型** |
| `,` | 分隔符，写在 `)` 和次数之间。可省略 |
| `+` | 次数：`*` 零到多、`+` 一到多、`?` 零或一（**不能带分隔符**） |

片段类型是固定清单，不是 Rust 类型：
`expr` `ident` `ty` `literal` `pat` `stmt` `block` `tt`

**声明宏跑在类型检查之前** —— 展开时编译器还不知道 `1` 是 `i32` 还是 `i64`，只知道「这是个表达式」。

### 最终实现

```rust
#[macro_export]
macro_rules! my_vec {
    () => {
        ::std::vec::Vec::new()
    };

    ($elem:expr; $count:expr) => {
        ::std::iter::repeat_n($elem, $count).collect::<::std::vec::Vec<_>>()
    };

    ($($x:expr),+ $(,)?) => {
        ::std::vec::Vec::from([$($x),+])
    };
}
```

### 踩过的坑

| 症状 | 病因 |
|---|---|
| `no rules expected 1`，提示 `while trying to match expr` | 写成了 `$(expr:isize)`。`$` 要贴在自己起的名字上；冒号后是片段类型，不是 Rust 类型 |
| 建的 Vec 每次都被重置 | `let mut v = ...` 被放进了重复组。重复组只装「每项都做一遍」的代码 |
| `macro expansion ignores v` | 展开体在表达式位置必须**是个表达式**。用块表达式包起来 |
| `meta-variable repeats with different Kleene operator` | 模式侧 `+`、展开侧 `*`，两边要统一 |
| `my_vec![5]` 静默返回空 Vec | 空规则写成了 `$($x:expr)?`（0 或 1 个），它把单元素调用也吞了。空规则的模式就是空 `()` |
| `repeat_n(4, 7)` 参数传反 | 元变量叫 `$x` / `$n` 看不出谁是谁。**取说人话的名字**，传错捕获项编译器不报错 |
| 调用方有自己的 `Vec` 就崩 | 路径不卫生。见下 |

### 卫生性：只覆盖局部变量

| 展开体里的东西 | 卫生吗 | 后果 |
|---|---|---|
| `let v = ...` 局部变量 | 是 | 宏内的 `v` 和调用方的 `v` 互不干扰 |
| `Vec`、`String`、函数名等路径 | **否** | 在调用方作用域解析，会被同名东西抢走 |

所以宏里引用外部名字一律写绝对路径：

- 标准库 → `::std::` 开头
- **本 crate 自己的东西 → `$crate::`**，展开时自动变成调用方走得通的路径

### 展开侧的重复也能带分隔符

```rust
$( v.push($x); )+   →   v.push(1); v.push(2); v.push(3);
$( $x ),+           →   1, 2, 3       ← 最后一项后面没有逗号
```

尾逗号 `$(,)?` 必须**另起一个重复组**放在模式末尾。写成 `$($x:expr),+,?` 会被解析成「分隔符 `,` + 次数 `?`」，而 `?` 不允许带分隔符。

---

## 二、过程宏

### 和声明宏的根本区别

声明宏是「模式 → 模板」，写的是**规则**。
过程宏是**编译期运行的普通 Rust 函数**，写的是**代码**。

```
proc_macro::TokenStream          编译器递进来（不含 #[derive(...)] 那一行）
        │  syn::parse            ① 解析
        ▼
  syn::DeriveInput
        │  普通 Rust 代码         ② 变换
        ▼
   要生成的代码片段
        │  quote!                ③ 生成
        ▼
proc_macro::TokenStream          **追加**在原类型旁边（derive 宏只增不改）
```

三个角色：**syn** 解析、**quote** 生成、**proc-macro2** 是不依赖编译器的 `TokenStream` 实现。
`proc_macro::TokenStream` 只在编译器调用时才存在，普通单测里构造不出来，所以内部逻辑一律用 proc-macro2，只在函数最外层做转换。

### 为什么必须单独一个 crate

编译器要先把过程宏编译成宿主机上的动态库，加载进自己的进程再调用它 —— 不可能一边编译一个 crate、一边加载这个还没编译完的 crate 来处理它自己。

```toml
[lib]
proc-macro = true    # 打开后只能导出宏，不能导出普通函数和类型
```

### `DeriveInput` 的形状

```
DeriveInput
├── attrs      打在类型上的 #[...]
├── vis        可见性
├── ident      类型名          ← 最常用
├── generics   泛型 / where
└── data       Data::Struct / Data::Enum / Data::Union
                └── variants   Punctuated<Variant, Token![,]>
                    ├── ident        变体名
                    ├── fields       Fields::Unnamed / Unit / Named
                    └── discriminant
```

`Fields` 三选一，对应三种变体写法：

| 源码 | syn 解析成 |
|---|---|
| `Int(i64)` | `Fields::Unnamed`，`Field.ident` 是 `None` |
| `Nothing` | `Fields::Unit` |
| `Pair { a: i64 }` | `Fields::Named`，`Field.ident` 是 `Some(..)` |

**类型嵌套很深但不用碰**：`Type::Path { path: Path { segments: [...] } }` 直接 `#ty` 插值即可，syn 的 AST 节点都实现了 `ToTokens`，会自己渲染回代码。看到深嵌套就想写递归是弯路。

### `quote!` 和声明宏是同一个思路

| 声明宏 | `quote!` |
|---|---|
| `$x` | `#x` |
| `$( ... )*` | `#( ... )*` |

### 最终实现

```rust
#[proc_macro_derive(TypeName)]
pub fn type_name_macros(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;
    let name_str = name.to_string();
    quote::quote! {
        impl #name {
            fn type_name() -> &'static str {
                #name_str
            }
        }
    }
    .into()
}
```

固定套路：函数必须 `pub`、必须直接写在 `lib.rs` 里（放子模块编译器找不到）；
`#[proc_macro_derive(X)]` 里的 `X` 才是用户写 `#[derive(X)]` 的名字，函数叫什么无所谓。

---

## 三、工具与环境坑

### `cargo expand` —— 最重要的一个

```bash
cargo expand --lib --tests decl::tests::list_form
cargo expand --lib enum_from
```

卡住的第一反应是跑它看展开结果，别盯着模式猜。

### 想看过程宏的输入长什么样

过程宏是编译期运行的普通程序，直接 `eprintln!("{input:#?}")`，输出在 `cargo build` 的 stderr 里。
需要给 syn 开 `features = ["extra-traits"]`，它默认不实现 `Debug`。

**没看到输出是被缓存挡了**：`touch` 一下使用宏的文件再 build。

### rust-analyzer 必须手动重启

对 r-a 来说过程宏是一个**已编译的动态库**，它 dlopen 进独立的 proc-macro-srv 进程。
改了宏源码，硬盘上的 dylib 更新了，进程里那份还是旧的 —— 动态库没法可靠热替换。
新增一个 `#[proc_macro_derive]` 更是必崩，旧 dylib 里根本没那个导出符号。

顺序不能反：

1. 先 `cargo build` 把新 dylib 编出来
2. 再执行 `rust-analyzer: Restart Server`

**只有改宏定义时才要重启**，改使用宏的代码不用。

写过程宏期间，**把编辑器的红线当噪音，以 `cargo build` 的输出为准**。

### rustfmt 不管宏体

rustfmt 只在宏体能解析成合法 Rust 代码时才格式化它。一旦出现 `$x` / `$(...)`，它解析失败就整个跳过，模式侧和展开侧一起放弃。`format_macro_matchers` 也救不了。

排版全靠自觉，通行写法是 `$($x:expr),+ $(,)?`，`),` 和 `+` 之间不留空格。

### TOML 括号不匹配，报错在下一行

```toml
syn = { version = "3", features = ["extra-traits"]     # 少一个 }
quote = "1"                                            # ← 报错指这里
```

inline table 没闭合，解析器以为还在 `{ }` 里，把下一行当成表里的下一个键。看到这类报错先往上看一行。

### 连字符 vs 下划线

| 位置 | 写法 |
|---|---|
| `Cargo.toml` 里的包名 | `macros-derive` 连字符 |
| Rust 代码里引用 | `macros_derive` 下划线 |

Rust 标识符不允许连字符，cargo 自动转换。`[workspace.dependencies]` 的键名就是包名，必须和 `[package] name` 一字不差。

---

## 四、方法论（比语法值钱）

1. **倒推法写宏**：先手写出「我希望它展开成什么代码」，再把变化的部分换成元变量，最后才套重复组。反过来「凑模板 → 看编译器脸色」走不通。
2. **卡住就 `cargo expand`**。
3. **编辑器补全在宏的展开侧帮不上忙** —— 那里不是 Rust 代码，是生成 Rust 代码的模板，补全只会帮你把语法凑合法，不知道你想干什么。

---

## 五、三个月后回来看什么

**值得记住的（不会忘的索引）**

- 宏在类型检查之前运行，它只认语法成分不认类型
- 路径不卫生 → 写 `::std::` 和 `$crate::`
- 过程宏是编译期运行的函数，三段式：syn 解析 → 变换 → quote 生成
- 卡住就 `cargo expand`；改了过程宏要重启 rust-analyzer

**可以放心忘掉的**

- 片段类型的完整清单（用时查本文档）
- syn 的具体 API（`DeriveInput` 有哪些字段、`Fields` 怎么匹配）
- darling 的属性解析写法（本次直接跳过，需要时再看）

**没做的练习**（同一套语法的排列组合，需要时再补）

- `my_try!` / `my_ready!`
- 完整的 `#[derive(EnumFrom)]`：遍历 `variants`、筛出单字段元组变体、为每个生成 `impl From<T>`。
  骨架和 `TypeName` 完全一样，只多了一层循环和一次筛选
